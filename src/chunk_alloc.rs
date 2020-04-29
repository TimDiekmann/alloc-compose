use crate::Owns;
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// Allocate memory with a multiple size of the provided chunk size.
///
/// # Examples
///
/// ```rust
/// #![feature(allocator_api)]
///
/// use alloc_compose::{ChunkAlloc, Region};
/// use std::alloc::{AllocInit, AllocRef, Global, Layout};
///
/// let mut data = [0; 64];
/// let mut alloc = ChunkAlloc::<_, 64>(Region::new(&mut data));
/// let memory = alloc.alloc(Layout::new::<[u8; 16]>(), AllocInit::Uninitialized)?;
/// assert_eq!(memory.size % 32, 0);
/// assert!(memory.size >= 32);
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// When growing or shrinking the memory, `ChunkAlloc` will try to alter
/// the memory in place before delegating to the underlying allocator.
///
/// ```rust
/// # #![feature(allocator_api)]
/// # use alloc_compose::{ChunkAlloc, Region};
/// # use std::alloc::{AllocInit, AllocRef, Global, Layout};
/// # let mut data = [0; 64];
/// # let mut alloc = ChunkAlloc::<_, 64>(Region::new(&mut data));
/// # let memory = alloc.alloc(Layout::new::<[u8; 16]>(), AllocInit::Uninitialized)?;
/// use std::alloc::ReallocPlacement;
/// let memory = unsafe {
///     alloc.grow(
///         memory.ptr,
///         Layout::new::<[u8; 16]>(),
///         24,
///         ReallocPlacement::InPlace,
///         AllocInit::Uninitialized,
///     )?
/// };
/// assert_eq!(memory.size % 32, 0);
/// assert!(memory.size >= 32);
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ChunkAlloc<A, const SIZE: usize>(pub A);

impl<A, const SIZE: usize> ChunkAlloc<A, SIZE> {
    const fn assert_alignment() {
        assert!(usize::is_power_of_two(SIZE), "SIZE must be a power of two");
    }

    const fn next_multiple(size: usize) -> usize {
        ((size + SIZE - 1) / SIZE) * SIZE
    }
}

unsafe impl<A: AllocRef, const SIZE: usize> AllocRef for ChunkAlloc<A, SIZE> {
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        Self::assert_alignment();
        self.0.alloc(
            unsafe {
                Layout::from_size_align_unchecked(
                    Self::next_multiple(layout.size()),
                    layout.align(),
                )
            },
            init,
        )
    }
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.0.dealloc(
            ptr,
            Layout::from_size_align_unchecked(Self::next_multiple(layout.size()), layout.align()),
        )
    }
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        let next_multiple = Self::next_multiple(layout.size());
        if new_size <= next_multiple {
            return Ok(MemoryBlock {
                ptr,
                size: next_multiple,
            });
        }

        self.0.grow(
            ptr,
            Layout::from_size_align_unchecked(next_multiple, layout.align()),
            Self::next_multiple(new_size),
            placement,
            init,
        )
    }
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        let next_multiple = Self::next_multiple(layout.size());
        let previous_multiple = next_multiple - SIZE;
        if new_size > previous_multiple {
            return Ok(MemoryBlock {
                ptr,
                size: next_multiple,
            });
        }

        self.0.shrink(
            ptr,
            Layout::from_size_align_unchecked(next_multiple, layout.align()),
            Self::next_multiple(new_size),
            placement,
        )
    }
}

impl<A: Owns, const SIZE: usize> Owns for ChunkAlloc<A, SIZE> {
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.0.owns(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkAlloc;
    use crate::helper;
    use std::alloc::{AllocInit, AllocRef, Layout, ReallocPlacement, System};

    #[test]
    fn alloc() {
        let mut alloc = helper::tracker(ChunkAlloc::<_, 64>(System));
        let memory = alloc
            .alloc(Layout::new::<u8>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");
        assert_eq!(memory.size % 64, 0);
        assert!(memory.size >= 64);

        unsafe {
            alloc.dealloc(memory.ptr, Layout::new::<u8>());
        }
    }

    #[test]
    fn dealloc() {
        let mut alloc = helper::tracker(ChunkAlloc::<_, 64>(System));

        unsafe {
            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.size % 64, 0);
            alloc.dealloc(memory.ptr, Layout::new::<[u8; 4]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.size % 64, 0);
            alloc.dealloc(memory.ptr, Layout::new::<[u8; 32]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.size % 64, 0);
            alloc.dealloc(memory.ptr, Layout::new::<[u8; 64]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.size % 64, 0);
            alloc.dealloc(memory.ptr, Layout::new::<[u8; 64]>());
        }
    }

    #[test]
    fn grow() {
        let mut alloc = helper::tracker(ChunkAlloc::<_, 64>(System));

        unsafe {
            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.size % 64, 0);

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 4]>(),
                    8,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 8 bytes");
            assert_eq!(memory.size % 64, 0);
            assert!(memory.size >= 64);

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 8]>(),
                    64,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 64 bytes");
            assert_eq!(memory.size % 64, 0);
            assert!(memory.size >= 64);

            alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 64]>(),
                    65,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect_err("Could grow to 65 bytes in place");

            alloc.dealloc(memory.ptr, Layout::new::<[u8; 64]>());
        }
    }

    #[test]
    fn shrink() {
        let mut alloc = helper::tracker(ChunkAlloc::<_, 64>(System));

        unsafe {
            let memory = alloc
                .alloc(Layout::new::<[u8; 128]>(), AllocInit::Uninitialized)
                .expect("Could not allocate 128 bytes");
            assert_eq!(memory.size % 64, 0);

            let memory = alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 128]>(),
                    100,
                    ReallocPlacement::InPlace,
                )
                .expect("Could not shrink to 100 bytes");
            assert_eq!(memory.size % 64, 0);
            assert!(memory.size >= 128);

            let memory = alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 100]>(),
                    65,
                    ReallocPlacement::InPlace,
                )
                .expect("Could not shrink to 65 bytes");
            assert_eq!(memory.size % 64, 0);
            assert!(memory.size >= 128);

            alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 65]>(),
                    64,
                    ReallocPlacement::InPlace,
                )
                .expect_err("Could shrink to 64 bytes in place");

            alloc.dealloc(memory.ptr, Layout::new::<[u8; 65]>());
        }
    }
}
