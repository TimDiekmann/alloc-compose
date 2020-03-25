use crate::{grow, Owns};
use alloc::alloc::{AllocErr, AllocInit, AllocRef, MemoryBlock, ReallocPlacement};
use core::alloc::Layout;

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
    fn alloc(self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        match self.primary.alloc(layout, init) {
            primary @ Ok(_) => primary,
            Err(_) => self.fallback.alloc(layout, init),
        }
    }

    unsafe fn dealloc(self, memory: MemoryBlock) {
        if self.primary.owns(&memory) {
            self.primary.dealloc(memory)
        } else {
            self.fallback.dealloc(memory)
        }
    }

    unsafe fn grow(
        self,
        memory: &mut MemoryBlock,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<(), AllocErr> {
        if self.primary.owns(memory) {
            if self.primary.grow(memory, new_size, placement, init).is_ok() {
                Ok(())
            } else {
                grow(
                    self.primary,
                    self.fallback,
                    memory,
                    new_size,
                    placement,
                    init,
                )
            }
        } else {
            self.fallback.grow(memory, new_size, placement, init)
        }
    }

    unsafe fn shrink(
        self,
        memory: &mut MemoryBlock,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<(), AllocErr> {
        if self.primary.owns(memory) {
            self.primary.shrink(memory, new_size, placement)
        } else {
            self.fallback.shrink(memory, new_size, placement)
        }
    }
}

impl<Primary, Fallback> Owns for FallbackAlloc<Primary, Fallback>
where
    Primary: Owns,
    Fallback: Owns,
{
    fn owns(&self, memory: &MemoryBlock) -> bool {
        self.primary.owns(memory) || self.fallback.owns(memory)
    }
}
