use crate::{grow, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// An allocator equivalent of an "or" operator in algebra.
///
/// An allocation request is first attempted with the `Primary` allocator. If that fails, the
/// request is forwarded to the `Fallback` allocator. All other requests are dispatched
/// appropriately to one of the two allocators.
///
/// A `Fallback` is useful for fast, special-purpose allocators backed up by general-purpose
/// allocators like [`Global`] or [`System`].
///
/// [`Global`]: https://doc.rust-lang.org/alloc/alloc/struct.Global.html
/// [`System`]: https://doc.rust-lang.org/std/alloc/struct.System.html
///
/// # Example
///
/// ```rust
/// #![feature(allocator_api)]
///
/// use alloc_compose::{Fallback, Owns, Region};
/// use std::{
///     alloc::{AllocInit, AllocRef, Layout, System},
///     mem::MaybeUninit,
/// };
///
/// let mut data = [MaybeUninit::new(0); 32];
/// let mut alloc = Fallback {
///     primary: Region::new(&mut data),
///     secondary: System,
/// };
///
/// let small_memory = alloc.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
/// let big_memory = alloc.alloc(Layout::new::<[u32; 64]>(), AllocInit::Uninitialized)?;
///
/// assert!(alloc.primary.owns(small_memory));
/// assert!(!alloc.primary.owns(big_memory));
///
/// unsafe {
///     // `big_memory` was allocated from `System`, we can dealloc it directly
///     System.dealloc(big_memory.ptr, Layout::new::<[u32; 64]>());
///     alloc.dealloc(small_memory.ptr, Layout::new::<u32>());
/// };
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Fallback<Primary, Secondary> {
    /// The primary allocator
    pub primary: Primary,
    /// The fallback allocator
    pub secondary: Secondary,
}

unsafe impl<Primary, Secondary> AllocRef for Fallback<Primary, Secondary>
where
    Primary: AllocRef + Owns,
    Secondary: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        match self.primary.alloc(layout, init) {
            primary @ Ok(_) => primary,
            Err(_) => self.secondary.alloc(layout, init),
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.primary.dealloc(ptr, layout)
        } else {
            self.secondary.dealloc(ptr, layout)
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
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            if let Ok(memory) = self.primary.grow(ptr, layout, new_size, placement, init) {
                Ok(memory)
            } else {
                grow(
                    &mut self.primary,
                    &mut self.secondary,
                    ptr,
                    layout,
                    new_size,
                    placement,
                    init,
                )
            }
        } else {
            self.secondary.grow(ptr, layout, new_size, placement, init)
        }
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.primary.shrink(ptr, layout, new_size, placement)
        } else {
            self.secondary.shrink(ptr, layout, new_size, placement)
        }
    }
}

impl<Primary, Secondary> Owns for Fallback<Primary, Secondary>
where
    Primary: Owns,
    Secondary: Owns,
{
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.primary.owns(memory) || self.secondary.owns(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::Fallback;
    use crate::{helper, Owns, Region};
    use std::{
        alloc::{AllocInit, AllocRef, Layout, ReallocPlacement, System},
        mem::MaybeUninit,
    };

    #[test]
    fn alloc() {
        let mut data = [MaybeUninit::new(0); 32];
        let mut alloc = Fallback {
            primary: helper::tracker(Region::new(&mut data)),
            secondary: helper::tracker(System),
        };

        let small_memory = alloc
            .alloc(Layout::new::<u32>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let big_memory = alloc
            .alloc(Layout::new::<[u8; 64]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");

        assert!(alloc.primary.owns(small_memory));
        assert!(!alloc.primary.owns(big_memory));
        unsafe {
            alloc.dealloc(small_memory.ptr, Layout::new::<u32>());
            alloc.dealloc(big_memory.ptr, Layout::new::<[u8; 64]>());
        };
    }

    #[test]
    fn grow() {
        let mut data = [MaybeUninit::new(0); 80];
        let mut alloc = Fallback {
            primary: helper::tracker(Region::new(&mut data)),
            secondary: helper::tracker(System),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 32 bytes");
        assert!(alloc.primary.owns(memory));

        unsafe {
            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 32]>(),
                    64,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 64 bytes");
            assert!(alloc.primary.owns(memory));
            assert_eq!(memory.size, 64);

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 64]>(),
                    80,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 80 bytes");
            assert!(alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 80]>(),
                    96,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 96 bytes");
            assert!(!alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 96]>(),
                    128,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 128 bytes");
            assert!(!alloc.primary.owns(memory));

            alloc.dealloc(memory.ptr, Layout::new::<[u8; 128]>());
        };
    }

    #[test]
    fn shrink() {
        let mut data = [MaybeUninit::new(0); 80];
        let mut alloc = Fallback {
            primary: helper::tracker(Region::new(&mut data)),
            secondary: helper::tracker(System),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 64]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");
        assert!(alloc.primary.owns(memory));

        unsafe {
            let memory = alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 64]>(),
                    32,
                    ReallocPlacement::MayMove,
                )
                .expect("Could not shrink to 32 bytes");
            assert!(alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 32]>(),
                    128,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 128 bytes");
            assert!(!alloc.primary.owns(memory));

            let memory = alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 128]>(),
                    96,
                    ReallocPlacement::MayMove,
                )
                .expect("Could not shrink to 96 bytes");
            assert!(!alloc.primary.owns(memory));

            alloc.dealloc(memory.ptr, Layout::new::<[u8; 96]>());
        }
    }

    #[test]
    fn owns() {
        let mut data_1 = [MaybeUninit::new(0); 32];
        let mut data_2 = [MaybeUninit::new(0); 64];
        let mut alloc = Fallback {
            primary: Region::new(&mut data_1),
            secondary: Region::new(&mut data_2),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 32 bytes");
        assert!(alloc.primary.owns(memory));
        assert!(alloc.owns(memory));

        let memory = alloc
            .alloc(Layout::new::<[u8; 64]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");
        assert!(alloc.secondary.owns(memory));
        assert!(alloc.owns(memory));
    }
}
