use crate::Owns;
use alloc::alloc::{AllocErr, AllocInit, AllocRef, MemoryBlock, ReallocPlacement};
use core::alloc::Layout;

/// An emphatically empty implementation of `AllocRef`.
///
/// Although it has no direct use, it is useful as a "terminator" in composite allocators.
#[derive(Debug, Copy, Clone)]
pub struct NullAlloc;

unsafe impl AllocRef for NullAlloc {
    fn alloc(self, _layout: Layout, _init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        Err(AllocErr)
    }

    unsafe fn dealloc(self, _memory: MemoryBlock) {
        panic!("NullAlloc::dealloc should never be called as `alloc` always fails")
    }

    unsafe fn grow(
        self,
        _memory: &mut MemoryBlock,
        _new_size: usize,
        _placement: ReallocPlacement,
        _init: AllocInit,
    ) -> Result<(), AllocErr> {
        panic!("NullAlloc::grow should never be called as `alloc` always fails")
    }

    unsafe fn shrink(
        self,
        _memory: &mut MemoryBlock,
        _new_size: usize,
        _placement: ReallocPlacement,
    ) -> Result<(), AllocErr> {
        panic!("NullAlloc::shrink should never be called as `alloc` always fails")
    }
}

impl Owns for NullAlloc {
    fn owns(&self, _memory: &MemoryBlock) -> bool {
        false
    }
}
