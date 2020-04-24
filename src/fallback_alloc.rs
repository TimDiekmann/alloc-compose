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
/// A `FallbackAlloc` is useful for fast, special-purpose allocators backed up by general-purpose
/// allocators like [`Global`] or [`System`].
///
/// [`Global`]: https://doc.rust-lang.org/alloc/alloc/struct.Global.html
/// [`System`]: https://doc.rust-lang.org/std/alloc/struct.System.html
#[derive(Debug, Copy, Clone)]
pub struct FallbackAlloc<Primary, Fallback> {
    /// The primary allocator
    pub primary: Primary,
    /// The fallback allocator
    pub fallback: Fallback,
}

unsafe impl<Primary, Fallback> AllocRef for FallbackAlloc<Primary, Fallback>
where
    Primary: AllocRef + Owns,
    Fallback: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        match self.primary.alloc(layout, init) {
            primary @ Ok(_) => primary,
            Err(_) => self.fallback.alloc(layout, init),
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.primary.dealloc(ptr, layout)
        } else {
            self.fallback.dealloc(ptr, layout)
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
                    &mut self.fallback,
                    ptr,
                    layout,
                    new_size,
                    placement,
                    init,
                )
            }
        } else {
            self.fallback.grow(ptr, layout, new_size, placement, init)
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
            self.fallback.shrink(ptr, layout, new_size, placement)
        }
    }
}

impl<Primary, Fallback> Owns for FallbackAlloc<Primary, Fallback>
where
    Primary: Owns,
    Fallback: Owns,
{
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.primary.owns(memory) || self.fallback.owns(memory)
    }
}
