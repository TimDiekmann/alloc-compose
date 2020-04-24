use crate::{grow, shrink, Owns};
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
pub struct SegregateAlloc<Small, Large> {
    threshold: usize,
    pub small: Small,
    pub large: Large,
}

impl<Small: AllocRef, Large: AllocRef> SegregateAlloc<Small, Large> {
    fn clamp_memory(&self, memory: MemoryBlock) -> MemoryBlock {
        MemoryBlock {
            ptr: memory.ptr,
            size: cmp::max(memory.size, self.threshold),
        }
    }
}

unsafe impl<Small, Large> AllocRef for SegregateAlloc<Small, Large>
where
    Small: AllocRef,
    Large: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        if layout.size() <= self.threshold {
            let memory = self.small.alloc(layout, init)?;
            Ok(self.clamp_memory(memory))
        } else {
            self.large.alloc(layout, init)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() <= self.threshold {
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
        if layout.size() <= self.threshold {
            let memory = if new_size > self.threshold {
                grow(
                    &mut self.small,
                    &mut self.large,
                    ptr,
                    layout,
                    new_size,
                    placement,
                    init,
                )?
            } else {
                self.small.grow(ptr, layout, new_size, placement, init)?
            };
            Ok(self.clamp_memory(memory))
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
        if layout.size() <= self.threshold {
            let memory = self.small.shrink(ptr, layout, new_size, placement)?;
            Ok(self.clamp_memory(memory))
        } else if new_size <= self.threshold {
            // Move ownership to `self.small`
            let memory = shrink(
                &mut self.large,
                &mut self.small,
                ptr,
                layout,
                new_size,
                placement,
            )?;
            Ok(self.clamp_memory(memory))
        } else {
            self.large.shrink(ptr, layout, new_size, placement)
        }
    }
}

impl<Small, Large> Owns for SegregateAlloc<Small, Large>
where
    Small: Owns,
    Large: Owns,
{
    fn owns(&self, memory: MemoryBlock) -> bool {
        if memory.size <= self.threshold {
            self.small.owns(memory)
        } else {
            self.large.owns(memory)
        }
    }
}
