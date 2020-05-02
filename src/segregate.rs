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
pub struct Segregate<Small, Large, const THRESHOLD: usize> {
    pub small: Small,
    pub large: Large,
}

impl<Small: AllocRef, Large: AllocRef, const THRESHOLD: usize> Segregate<Small, Large, THRESHOLD> {
    fn clamp_memory(memory: MemoryBlock) -> MemoryBlock {
        MemoryBlock {
            ptr: memory.ptr,
            size: cmp::max(memory.size, THRESHOLD),
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
            let memory = if new_size > THRESHOLD {
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
            Ok(Self::clamp_memory(memory))
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
