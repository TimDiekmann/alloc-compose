use crate::{AllocateAll, Owns, ReallocateInPlace};
use core::{
    alloc::{AllocError, AllocRef, Layout},
    ptr::NonNull,
};

/// An emphatically empty implementation of `AllocRef`.
///
/// Although it has no direct use, it is useful as a "terminator" in composite allocators
/// or for disabling the global allocator.
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
///
/// ## Disabling the global allocator
///
/// ```rust, no_run
/// use alloc_compose::Null;
///
/// #[global_allocator]
/// static A: Null = Null;
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Null;

unsafe impl AllocRef for Null {
    /// Will always return `Err(AllocErr)`.
    fn alloc(&self, _layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    /// Will always return `Err(AllocErr)`.
    fn alloc_zeroed(&self, _layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    /// Must not be called, as allocation always fails.
    unsafe fn dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {
        unreachable!("Null::dealloc must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn grow(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unreachable!("Null::grow must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn grow_zeroed(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unreachable!("Null::grow_zeroed must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn shrink(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unreachable!("Null::shrink must never be called as allocation always fails")
    }
}

unsafe impl AllocateAll for Null {
    fn allocate_all(&self) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    fn allocate_all_zeroed(&self) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    fn deallocate_all(&self) {}

    fn capacity(&self) -> usize {
        0
    }

    fn capacity_left(&self) -> usize {
        0
    }
}

unsafe impl ReallocateInPlace for Null {
    /// Must not be called, as allocation always fails.
    unsafe fn grow_in_place(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<usize, AllocError> {
        unreachable!("Null::grow_in_place must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn grow_in_place_zeroed(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<usize, AllocError> {
        unreachable!("Null::grow_in_place_zeroed must never be called as allocation always fails")
    }

    /// Must not be called, as allocation always fails.
    unsafe fn shrink_in_place(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<usize, AllocError> {
        unreachable!("Null::shrink_in_place must never be called as allocation always fails")
    }
}

impl Owns for Null {
    /// Will always return `false.
    fn owns(&self, _memory: NonNull<[u8]>) -> bool {
        false
    }
}

impl_global_alloc!(Null);

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
        assert!(Null.allocate_all().is_err());
        assert!(Null.allocate_all_zeroed().is_err());
        assert_eq!(Null.capacity(), 0);
        assert_eq!(Null.capacity_left(), 0);
        Null.deallocate_all();
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow() {
        unsafe {
            let _ = Null.grow(
                NonNull::dangling(),
                Layout::new::<()>(),
                Layout::new::<()>(),
            );
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow_zeroed() {
        unsafe {
            let _ = Null.grow_zeroed(
                NonNull::dangling(),
                Layout::new::<()>(),
                Layout::new::<()>(),
            );
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow_in_place() {
        unsafe {
            let _ = Null.grow_in_place(
                NonNull::dangling(),
                Layout::new::<()>(),
                Layout::new::<()>(),
            );
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn grow_in_place_zeroed() {
        unsafe {
            let _ = Null.grow_in_place_zeroed(
                NonNull::dangling(),
                Layout::new::<()>(),
                Layout::new::<()>(),
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
                Layout::new::<()>(),
            );
        };
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn shrink_in_place() {
        unsafe {
            let _ = Null.shrink_in_place(
                NonNull::dangling(),
                Layout::new::<()>(),
                Layout::new::<()>(),
            );
        };
    }

    #[test]
    fn owns() {
        assert!(!Null.owns(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)));
    }

    #[test]
    fn debug() {
        assert_eq!(alloc::format!("{:?}", Null), "Null");
    }
}
