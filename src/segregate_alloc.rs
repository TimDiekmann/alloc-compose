use crate::{grow, shrink, Owns};
use alloc::alloc::{AllocErr, AllocInit, AllocRef, MemoryBlock, ReallocPlacement};
use core::alloc::Layout;

/// Dispatches calls to `AllocRef` between two allocators depending on the size allocated.
///
/// All allocations smaller than or equal to `threshold` will be dispatched to `Small`. The others
/// will go to `Large`.
#[derive(Debug, Copy, Clone)]
pub struct SegregateAlloc<Small, Large> {
    threshold: usize,
    pub small: Small,
    pub large: Large,
}

impl<Small: AllocRef, Large: AllocRef> SegregateAlloc<Small, Large> {
    fn clamp_memory(&self, memory: &mut MemoryBlock) {
        if memory.size() > self.threshold {
            unsafe {
                *memory = MemoryBlock::new(
                    memory.ptr(),
                    Layout::from_size_align_unchecked(self.threshold, memory.align()),
                );
            }
        }
    }
}

unsafe impl<Small, Large> AllocRef for SegregateAlloc<Small, Large>
where
    Small: AllocRef,
    Large: AllocRef,
{
    fn alloc(self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        if layout.size() <= self.threshold {
            let mut memory = self.small.alloc(layout, init)?;
            self.clamp_memory(&mut memory);
            Ok(memory)
        } else {
            self.large.alloc(layout, init)
        }
    }

    unsafe fn dealloc(self, memory: MemoryBlock) {
        if memory.size() <= self.threshold {
            self.small.dealloc(memory)
        } else {
            self.large.dealloc(memory)
        }
    }

    unsafe fn grow(
        self,
        memory: &mut MemoryBlock,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<(), AllocErr> {
        if memory.size() <= self.threshold {
            if new_size > self.threshold {
                grow(self.small, self.large, memory, new_size, placement, init)?;
            } else {
                self.small.grow(memory, new_size, placement, init)?;
            }
            self.clamp_memory(memory);
            Ok(())
        } else {
            self.large.grow(memory, new_size, placement, init)
        }
    }

    unsafe fn shrink(
        self,
        memory: &mut MemoryBlock,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<(), AllocErr> {
        if memory.size() <= self.threshold {
            self.small.shrink(memory, new_size, placement)?;
        } else if new_size <= self.threshold {
            shrink(self.large, self.small, memory, new_size, placement)?;
        } else {
            self.large.shrink(memory, new_size, placement)?;
        }
        self.clamp_memory(memory);
        Ok(())
    }
}

impl<Small, Large> Owns for SegregateAlloc<Small, Large>
where
    Small: Owns,
    Large: Owns,
{
    fn owns(&self, memory: &MemoryBlock) -> bool {
        if memory.size() <= self.threshold {
            self.small.owns(memory)
        } else {
            self.large.owns(memory)
        }
    }
}
