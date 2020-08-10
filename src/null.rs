use crate::{AllocAll, Owns, ReallocInPlace};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
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
/// use std::alloc::{AllocRef, Global, Layout};
///
/// let memory = Null.alloc(Layout::new::<u32>());
/// assert!(memory.is_err())
/// ```
///
/// Even if a zero-sized allocation is requested:
///
/// ```rust
/// # #![feature(allocator_api)]
/// # use alloc_compose::Null;
/// # use std::alloc::{AllocRef, Global, Layout};
/// let memory = Null.alloc(Layout::new::<()>());
/// assert!(memory.is_err())
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Null;

unsafe impl AllocRef for Null {
    /// Will always return `Err(AllocErr)`.
    fn alloc(&mut self, _layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        Err(AllocErr)
    }

    /// Will always return `Err(AllocErr)`.
    fn alloc_zeroed(&mut self, _layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        Err(AllocErr)
    }

    /// Must not be called, as allocation always fails.
    unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        unreachable!("Null::dealloc must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn grow(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        unreachable!("Null::grow must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn grow_zeroed(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        unreachable!("Null::grow_zeroed must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn shrink(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        unreachable!("Null::shrink must never be called as allocation always fails")
    }
}

unsafe impl AllocAll for Null {
    fn alloc_all(&mut self, _layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        Err(AllocErr)
    }

    fn alloc_all_zeroed(&mut self, _layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
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

unsafe impl ReallocInPlace for Null {
    /// Must not be called, as allocation always fails.
    unsafe fn grow_in_place(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
    ) -> Result<usize, AllocErr> {
        unreachable!("Null::grow_in_place must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn grow_in_place_zeroed(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
    ) -> Result<usize, AllocErr> {
        unreachable!("Null::grow_in_place_zeroed must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn shrink_in_place(
        &mut self,
        _ptr: NonNull<u8>,
        _layout: Layout,
        _new_size: usize,
    ) -> Result<usize, AllocErr> {
        unreachable!("Null::shrink_in_place must never be called as allocation always fails")
    }
}

impl Owns for Null {
    /// Will always return `false.
    fn owns(&self, _memory: NonNull<[u8]>) -> bool {
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
    fn alloc() {
        assert!(Null.alloc(Layout::new::<u32>()).is_err());
        assert!(Null.alloc_zeroed(Layout::new::<u32>()).is_err());
        assert!(Null.alloc_all(Layout::new::<u32>()).is_err());
        assert!(Null.alloc_all_zeroed(Layout::new::<u32>()).is_err());
        assert_eq!(Null.capacity(), 0);
        assert_eq!(Null.capacity_left(), 0);
        Null.dealloc_all();
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow() {
        unsafe {
            let _ = Null.grow(NonNull::dangling(), Layout::new::<()>(), 0);
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow_zeroed() {
        unsafe {
            let _ = Null.grow_zeroed(NonNull::dangling(), Layout::new::<()>(), 0);
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow_in_place() {
        unsafe {
            let _ = Null.grow_in_place(NonNull::dangling(), Layout::new::<()>(), 0);
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow_in_place_zeroed() {
        unsafe {
            let _ = Null.grow_in_place_zeroed(NonNull::dangling(), Layout::new::<()>(), 0);
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn shrink() {
        unsafe {
            let _ = Null.shrink(NonNull::dangling(), Layout::new::<()>(), 0);
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn shrink_in_place() {
        unsafe {
            let _ = Null.shrink_in_place(NonNull::dangling(), Layout::new::<()>(), 0);
        };
    }

    #[test]
    fn owns() {
        assert!(!Null.owns(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)));
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", Null), "Null");
    }
}
