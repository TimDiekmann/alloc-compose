use crate::{AllocAll, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// An emphatically empty implementation of `AllocRef`.
///
/// Although it has no direct use, it is useful as a "terminator" in composite allocators.
///
/// # Examples
///
/// The `Null` will always return `Err`:
///
/// ```rust
/// #![feature(allocator_api)]
///
/// use alloc_compose::Null;
/// use std::alloc::{AllocInit, AllocRef, Global, Layout};
///
/// let memory = Null.alloc(Layout::new::<u32>(), AllocInit::Uninitialized);
/// assert!(memory.is_err())
/// ```
///
/// Even if a zero-sized allocation is requested:
///
/// ```rust
/// # #![feature(allocator_api)]
/// # use alloc_compose::Null;
/// # use std::alloc::{AllocInit, AllocRef, Global, Layout};
/// let memory = Null.alloc(Layout::new::<()>(), AllocInit::Uninitialized);
/// assert!(memory.is_err())
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Null;

unsafe impl AllocRef for Null {
    /// Will always return `Err(AllocErr)`.
    fn alloc(&mut self, _layout: Layout, _init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        Err(AllocErr)
    }

    /// Must not be called, as `alloc` always fails.
    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        unreachable!("NullAlloc::dealloc must never be called as `alloc` always fails")
    }

    /// Must not be called, as `alloc` always fails.
    unsafe fn grow(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
        _placement: ReallocPlacement,
        _init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        unreachable!("NullAlloc::grow must never be called as `alloc` always fails")
    }

    /// Must not be called, as `alloc` always fails.
    unsafe fn shrink(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
        _placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        unreachable!("NullAlloc::shrink must never be called as `alloc` always fails")
    }
}

impl AllocAll for Null {
    fn alloc_all(&mut self, _layout: Layout, _init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        Err(AllocErr)
    }

    fn dealloc_all(&mut self) {}

    fn capacity(&self) -> usize {
        0
    }

    fn capacity_left(&self) -> usize {
        0
    }
}

impl Owns for Null {
    /// Will always return `false.
    fn owns(&self, _memory: MemoryBlock) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    #[should_panic(expected = "unreachable")]
    fn dealloc() {
        unsafe { Null.dealloc(NonNull::dangling(), Layout::new::<()>()) };
    }

    #[test]
    fn alloc_all() {
        assert!(
            Null.alloc_all(Layout::new::<u32>(), AllocInit::Uninitialized)
                .is_err()
        );
        assert_eq!(Null.capacity(), 0);
        assert_eq!(Null.capacity_left(), 0);
        Null.dealloc_all();
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow() {
        unsafe {
            let _ = Null.grow(
                NonNull::dangling(),
                Layout::new::<()>(),
                0,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            );
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn shrink() {
        unsafe {
            let _ = Null.shrink(
                NonNull::dangling(),
                Layout::new::<()>(),
                0,
                ReallocPlacement::MayMove,
            );
        };
    }

    #[test]
    fn owns() {
        assert!(!Null.owns(MemoryBlock {
            ptr: NonNull::dangling(),
            size: 0
        }));
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", Null), "Null");
    }
}
