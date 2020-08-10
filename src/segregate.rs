use crate::{
    helper::{grow_fallback, shrink_fallback, AllocInit},
    AllocAll,
    Owns,
};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
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
    fn clamped(ptr: NonNull<[u8]>) -> NonNull<[u8]> {
        NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), cmp::min(ptr.len(), THRESHOLD))
    }
}

unsafe impl<Small, Large, const THRESHOLD: usize> AllocRef for Segregate<Small, Large, THRESHOLD>
where
    Small: AllocRef,
    Large: AllocRef,
{
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.alloc(layout)?;
            Ok(Self::clamped(memory))
        } else {
            self.large.alloc(layout)
        }
    }

    fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.alloc_zeroed(layout)?;
            Ok(Self::clamped(memory))
        } else {
            self.large.alloc_zeroed(layout)
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
    ) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            if new_size > THRESHOLD {
                grow_fallback(
                    &mut self.small,
                    &mut self.large,
                    ptr,
                    layout,
                    new_size,
                    AllocInit::Uninitialized,
                )
            } else {
                let memory = self.small.grow(ptr, layout, new_size)?;
                Ok(Self::clamped(memory))
            }
        } else {
            self.large.grow(ptr, layout, new_size)
        }
    }

    unsafe fn grow_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            if new_size > THRESHOLD {
                grow_fallback(
                    &mut self.small,
                    &mut self.large,
                    ptr,
                    layout,
                    new_size,
                    AllocInit::Zeroed,
                )
            } else {
                let memory = self.small.grow_zeroed(ptr, layout, new_size)?;
                Ok(Self::clamped(memory))
            }
        } else {
            self.large.grow_zeroed(ptr, layout, new_size)
        }
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.shrink(ptr, layout, new_size)?;
            Ok(Self::clamped(memory))
        } else if new_size <= THRESHOLD {
            // Move ownership to `self.small`
            let memory = shrink_fallback(&mut self.large, &mut self.small, ptr, layout, new_size)?;
            Ok(Self::clamped(memory))
        } else {
            self.large.shrink(ptr, layout, new_size)
        }
    }
}

unsafe impl<Small, Large, const THRESHOLD: usize> AllocAll for Segregate<Small, Large, THRESHOLD>
where
    Small: AllocAll,
    Large: AllocAll,
{
    fn alloc_all(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.alloc_all(layout)?;
            Ok(Self::clamped(memory))
        } else {
            self.large.alloc_all(layout)
        }
    }

    fn alloc_all_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        if layout.size() <= THRESHOLD {
            let memory = self.small.alloc_all_zeroed(layout)?;
            Ok(Self::clamped(memory))
        } else {
            self.large.alloc_all(layout)
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
    fn owns(&self, ptr: NonNull<[u8]>) -> bool {
        if ptr.len() <= THRESHOLD {
            self.small.owns(ptr)
        } else {
            self.large.owns(ptr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Segregate;
    use crate::{AllocAll, Owns, Region};
    use core::{
        alloc::{AllocRef, Layout},
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
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        assert_eq!(mem.len(), 4);
        assert!(alloc.small.owns(mem));

        unsafe { alloc.dealloc(mem.as_non_null_ptr(), Layout::new::<[u8; 4]>()) };
        assert!(!alloc.owns(mem));

        let mem = alloc
            .alloc(Layout::new::<[u8; 32]>())
            .expect("Could not allocate 32 bytes");
        assert_eq!(mem.len(), 32);
        assert!(alloc.small.owns(mem));

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), alloc.capacity() - 32);

        let mem = alloc
            .alloc(Layout::new::<[u8; 33]>())
            .expect("Could not allocate 33 bytes");
        assert_eq!(mem.len(), 33);
        assert!(alloc.large.owns(mem));

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), alloc.capacity() - 32 - 33);

        unsafe {
            alloc.dealloc(mem.as_non_null_ptr(), Layout::new::<[u8; 33]>());
        }
        assert_eq!(alloc.capacity_left(), alloc.capacity() - 32);

        alloc.dealloc_all();
        assert_eq!(alloc.capacity(), alloc.capacity_left());

        let mem = alloc
            .alloc_all(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        assert!(alloc.small.owns(mem));
        assert_eq!(mem.len(), 32);

        assert_eq!(alloc.capacity(), 256);
        assert_eq!(alloc.capacity_left(), 128);

        let mem = alloc
            .alloc_all(Layout::new::<[u8; 33]>())
            .expect("Could not allocate 33 bytes");
        assert!(alloc.large.owns(mem));
        assert_eq!(mem.len(), 128);

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

        let mem = alloc.alloc(Layout::new::<[u8; 8]>()).unwrap();
        assert_eq!(mem.len(), 8);
        assert!(alloc.small.owns(mem));
        assert!(alloc.owns(mem));

        unsafe {
            let mem = alloc
                .grow(mem.as_non_null_ptr(), Layout::new::<[u8; 8]>(), 16)
                .unwrap();
            assert_eq!(mem.len(), 16);
            assert!(alloc.small.owns(mem));
            assert!(alloc.owns(mem));

            let mem = alloc
                .grow(mem.as_non_null_ptr(), Layout::new::<[u8; 8]>(), 32)
                .unwrap();
            assert_eq!(mem.len(), 32);
            assert!(alloc.small.owns(mem));
            assert!(alloc.owns(mem));

            let mem = alloc
                .grow(mem.as_non_null_ptr(), Layout::new::<[u8; 32]>(), 33)
                .unwrap();
            assert_eq!(mem.len(), 33);
            assert!(!alloc.small.owns(mem));
            assert!(alloc.large.owns(mem));
            assert!(alloc.owns(mem));

            let mem = alloc
                .grow(mem.as_non_null_ptr(), Layout::new::<[u8; 33]>(), 64)
                .unwrap();
            assert_eq!(mem.len(), 64);
            assert!(!alloc.small.owns(mem));
            assert!(alloc.large.owns(mem));
            assert!(alloc.owns(mem));
        }
    }
}
