use crate::{grow, shrink, AllocAll, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    cmp,
    ptr::NonNull,
};

/// Dispatches calls to `AllocRef` between two allocators depending on the size allocated.
///
/// All allocations smaller than or equal to `threshold` will be dispatched to `Small`. The others
/// will go to `Large`.
#[derive(Debug, Copy, Clone)]
pub struct Segregate<Small, Large, const THRESHOLD: usize> {
    pub small: Small,
    pub large: Large,
}

impl<Small, Large, const THRESHOLD: usize> Segregate<Small, Large, THRESHOLD> {
    fn clamp_memory(memory: MemoryBlock) -> MemoryBlock {
        MemoryBlock {
            ptr: memory.ptr,
            size: cmp::min(memory.size, THRESHOLD),
        }
    }
}

unsafe impl<Small, Large, const THRESHOLD: usize> AllocRef for Segregate<Small, Large, THRESHOLD>
where
    Small: AllocRef,
    Large: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.alloc(layout, init)?;
            Ok(Self::clamp_memory(memory))
        } else {
            self.large.alloc(layout, init)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() <= THRESHOLD {
            self.small.dealloc(ptr, layout)
        } else {
            self.large.dealloc(ptr, layout)
        }
    }

    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        if layout.size() <= THRESHOLD {
            if new_size > THRESHOLD {
                grow(
                    &mut self.small,
                    &mut self.large,
                    ptr,
                    layout,
                    new_size,
                    placement,
                    init,
                )
            } else {
                let memory = self.small.grow(ptr, layout, new_size, placement, init)?;
                Ok(Self::clamp_memory(memory))
            }
        } else {
            self.large.grow(ptr, layout, new_size, placement, init)
        }
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.shrink(ptr, layout, new_size, placement)?;
            Ok(Self::clamp_memory(memory))
        } else if new_size <= THRESHOLD {
            // Move ownership to `self.small`
            let memory = shrink(
                &mut self.large,
                &mut self.small,
                ptr,
                layout,
                new_size,
                placement,
            )?;
            Ok(Self::clamp_memory(memory))
        } else {
            self.large.shrink(ptr, layout, new_size, placement)
        }
    }
}
impl<Small, Large, const THRESHOLD: usize> AllocAll for Segregate<Small, Large, THRESHOLD>
where
    Small: AllocAll,
    Large: AllocAll,
{
    fn alloc_all(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.alloc_all(layout, init)?;
            Ok(Self::clamp_memory(memory))
        } else {
            self.large.alloc_all(layout, init)
        }
    }

    /// Deallocates all the memory the allocator had allocated.
    fn dealloc_all(&mut self) {
        self.small.dealloc_all();
        self.large.dealloc_all();
    }

    /// Returns the total capacity available in this allocator.
    fn capacity(&self) -> usize {
        self.small.capacity() + self.large.capacity()
    }

    /// Returns the free capacity left for allocating.
    fn capacity_left(&self) -> usize {
        self.small.capacity_left() + self.large.capacity_left()
    }
}

impl<Small, Large, const THRESHOLD: usize> Owns for Segregate<Small, Large, THRESHOLD>
where
    Small: Owns,
    Large: Owns,
{
    fn owns(&self, memory: MemoryBlock) -> bool {
        if memory.size <= THRESHOLD {
            self.small.owns(memory)
        } else {
            self.large.owns(memory)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Segregate;
    use crate::{AllocAll, Owns, Region};
    use core::{
        alloc::{AllocInit, AllocRef, Layout, ReallocPlacement},
        mem::MaybeUninit,
    };

    #[test]
    fn alloc() {
        let mut data_1 = [MaybeUninit::new(0); 128];
        let mut data_2 = [MaybeUninit::new(0); 128];

        let mut alloc: Segregate<_, _, 32> = Segregate {
            small: Region::new(&mut data_1),
            large: Region::new(&mut data_2),
        };

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), alloc.capacity());

        let mem = alloc
            .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        assert_eq!(mem.size, 4);
        assert!(alloc.small.owns(mem));

        unsafe { alloc.dealloc(mem.ptr, Layout::new::<[u8; 4]>()) };
        assert!(!alloc.owns(mem));

        let mem = alloc
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 32 bytes");
        assert_eq!(mem.size, 32);
        assert!(alloc.small.owns(mem));

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), alloc.capacity() - 32);

        let mem = alloc
            .alloc(Layout::new::<[u8; 33]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 33 bytes");
        assert_eq!(mem.size, 33);
        assert!(alloc.large.owns(mem));

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), alloc.capacity() - 32 - 33);

        unsafe {
            alloc.dealloc(mem.ptr, Layout::new::<[u8; 33]>());
        }
        assert_eq!(alloc.capacity_left(), alloc.capacity() - 32);

        alloc.dealloc_all();
        assert_eq!(alloc.capacity(), alloc.capacity_left());

        let mem = alloc
            .alloc_all(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        assert!(alloc.small.owns(mem));
        assert_eq!(mem.size, 32);

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), 128);

        let mem = alloc
            .alloc_all(Layout::new::<[u8; 33]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 33 bytes");
        assert!(alloc.large.owns(mem));
        assert_eq!(mem.size, 128);

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), 0);

        alloc.dealloc_all();

        assert_eq!(alloc.capacity_left(), alloc.capacity());
    }

    #[test]
    fn realloc() {
        let mut data_1 = [MaybeUninit::new(0); 128];
        let mut data_2 = [MaybeUninit::new(0); 128];

        let mut alloc: Segregate<_, _, 32> = Segregate {
            small: Region::new(&mut data_1),
            large: Region::new(&mut data_2),
        };

        let mem = alloc
            .alloc(Layout::new::<[u8; 8]>(), AllocInit::Uninitialized)
            .unwrap();
        assert_eq!(mem.size, 8);
        assert!(alloc.small.owns(mem));
        assert!(alloc.owns(mem));

        unsafe {
            let mem = alloc
                .grow(
                    mem.ptr,
                    Layout::new::<[u8; 8]>(),
                    16,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .unwrap();
            assert_eq!(mem.size, 16);
            assert!(alloc.small.owns(mem));
            assert!(alloc.owns(mem));

            let mem = alloc
                .grow(
                    mem.ptr,
                    Layout::new::<[u8; 8]>(),
                    32,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .unwrap();
            assert_eq!(mem.size, 32);
            assert!(alloc.small.owns(mem));
            assert!(alloc.owns(mem));

            let mem = alloc
                .grow(
                    mem.ptr,
                    Layout::new::<[u8; 32]>(),
                    33,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .unwrap();
            assert_eq!(mem.size, 33);
            assert!(!alloc.small.owns(mem));
            assert!(alloc.large.owns(mem));
            assert!(alloc.owns(mem));

            let mem = alloc
                .grow(
                    mem.ptr,
                    Layout::new::<[u8; 33]>(),
                    64,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .unwrap();
            assert_eq!(mem.size, 64);
            assert!(!alloc.small.owns(mem));
            assert!(alloc.large.owns(mem));
            assert!(alloc.owns(mem));
        }
    }
}
