use crate::{
    helper::{grow_fallback, AllocInit},
    Owns,
};
use core::{
    alloc::{AllocError, AllocRef, Layout},
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
/// #![feature(allocator_api, slice_ptr_get)]
///
/// use alloc_compose::{region::Region, Fallback, Owns};
/// use std::{
///     alloc::{AllocRef, Layout, System},
///     mem::MaybeUninit,
/// };
///
/// let mut data = [MaybeUninit::new(0); 32];
/// let mut alloc = Fallback {
///     primary: Region::new(&mut data),
///     secondary: System,
/// };
///
/// let small_memory = alloc.alloc(Layout::new::<u32>())?;
/// let big_memory = alloc.alloc(Layout::new::<[u32; 64]>())?;
///
/// assert!(alloc.primary.owns(small_memory));
/// assert!(!alloc.primary.owns(big_memory));
///
/// unsafe {
///     // `big_memory` was allocated from `System`, we can dealloc it directly
///     System.dealloc(big_memory.as_non_null_ptr(), Layout::new::<[u32; 64]>());
///     alloc.dealloc(small_memory.as_non_null_ptr(), Layout::new::<u32>());
/// };
/// # Ok::<(), core::alloc::AllocError>(())
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
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.primary.alloc(layout) {
            primary @ Ok(_) => primary,
            Err(_) => self.secondary.alloc(layout),
        }
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.primary.alloc_zeroed(layout) {
            primary @ Ok(_) => primary,
            Err(_) => self.secondary.alloc_zeroed(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        if self
            .primary
            .owns(NonNull::slice_from_raw_parts(ptr, layout.size()))
        {
            self.primary.dealloc(ptr, layout)
        } else {
            self.secondary.dealloc(ptr, layout)
        }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self
            .primary
            .owns(NonNull::slice_from_raw_parts(ptr, old_layout.size()))
        {
            if let Ok(memory) = self.primary.grow(ptr, old_layout, new_layout) {
                Ok(memory)
            } else {
                grow_fallback(
                    &self.primary,
                    &self.secondary,
                    ptr,
                    old_layout,
                    new_layout,
                    AllocInit::Uninitialized,
                )
            }
        } else {
            self.secondary.grow(ptr, old_layout, new_layout)
        }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self
            .primary
            .owns(NonNull::slice_from_raw_parts(ptr, old_layout.size()))
        {
            if let Ok(memory) = self.primary.grow_zeroed(ptr, old_layout, new_layout) {
                Ok(memory)
            } else {
                grow_fallback(
                    &self.primary,
                    &self.secondary,
                    ptr,
                    old_layout,
                    new_layout,
                    AllocInit::Zeroed,
                )
            }
        } else {
            self.secondary.grow_zeroed(ptr, old_layout, new_layout)
        }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self
            .primary
            .owns(NonNull::slice_from_raw_parts(ptr, old_layout.size()))
        {
            self.primary.shrink(ptr, old_layout, new_layout)
        } else {
            self.secondary.shrink(ptr, old_layout, new_layout)
        }
    }
}

impl<Primary, Secondary> Owns for Fallback<Primary, Secondary>
where
    Primary: Owns,
    Secondary: Owns,
{
    fn owns(&self, memory: NonNull<[u8]>) -> bool {
        self.primary.owns(memory) || self.secondary.owns(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::Fallback;
    use crate::{helper, region::Region, Chunk, Owns};
    use alloc::alloc::Global;
    use core::{
        alloc::{AllocRef, Layout},
        mem::MaybeUninit,
    };

    #[test]
    fn alloc() {
        let mut data = [MaybeUninit::new(0); 32];
        let alloc = Fallback {
            primary: helper::tracker(Region::new(&mut data)),
            secondary: helper::tracker(Global),
        };

        let small_memory = alloc
            .alloc(Layout::new::<u32>())
            .expect("Could not allocate 4 bytes");
        let big_memory = alloc
            .alloc(Layout::new::<[u8; 64]>())
            .expect("Could not allocate 64 bytes");

        assert!(alloc.primary.owns(small_memory));
        assert!(!alloc.primary.owns(big_memory));
        unsafe {
            alloc.dealloc(small_memory.as_non_null_ptr(), Layout::new::<u32>());
            alloc.dealloc(big_memory.as_non_null_ptr(), Layout::new::<[u8; 64]>());
        };
    }

    #[test]
    fn grow() {
        let mut data = [MaybeUninit::new(0); 80];
        let alloc = Fallback {
            primary: helper::tracker(Chunk::<Region, 64>(Region::new(&mut data))),
            secondary: helper::tracker(Global),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 32]>())
            .expect("Could not allocate 4 bytes");
        assert!(alloc.primary.owns(memory));

        unsafe {
            let memory = alloc
                .grow(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 32]>(),
                    Layout::new::<[u8; 64]>(),
                )
                .expect("Could not grow to 64 bytes");
            assert!(alloc.primary.owns(memory));
            assert_eq!(memory.len(), 64);

            let memory = alloc
                .grow(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 64]>(),
                    Layout::new::<[u8; 128]>(),
                )
                .expect("Could not grow to 128 bytes");
            assert!(!alloc.primary.owns(memory));

            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 128]>());
        };
    }

    #[test]
    fn shrink() {
        let mut data = [MaybeUninit::new(0); 80];
        let alloc = Fallback {
            primary: helper::tracker(Chunk::<Region, 64>(Region::new(&mut data))),
            secondary: helper::tracker(Global),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 64]>())
            .expect("Could not allocate 64 bytes");
        assert!(alloc.primary.owns(memory));

        unsafe {
            let memory = alloc
                .shrink(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 64]>(),
                    Layout::new::<[u8; 32]>(),
                )
                .expect("Could not shrink to 32 bytes");
            assert!(alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 32]>(),
                    Layout::new::<[u8; 128]>(),
                )
                .expect("Could not grow to 128 bytes");
            assert!(!alloc.primary.owns(memory));

            let memory = alloc
                .shrink(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 128]>(),
                    Layout::new::<[u8; 96]>(),
                )
                .expect("Could not shrink to 96 bytes");
            assert!(!alloc.primary.owns(memory));

            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 96]>());
        }
    }

    #[test]
    fn owns() {
        let mut data_1 = [MaybeUninit::new(0); 32];
        let mut data_2 = [MaybeUninit::new(0); 64];
        let alloc = Fallback {
            primary: Region::new(&mut data_1),
            secondary: Region::new(&mut data_2),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 32]>())
            .expect("Could not allocate 32 bytes");
        assert!(alloc.primary.owns(memory));
        assert!(alloc.owns(memory));

        let memory = alloc
            .alloc(Layout::new::<[u8; 64]>())
            .expect("Could not allocate 64 bytes");
        assert!(alloc.secondary.owns(memory));
        assert!(alloc.owns(memory));
    }
}
