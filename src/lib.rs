#![cfg_attr(not(test), no_std)]
#![cfg_attr(doc, feature(doc_cfg, external_doc))]
#![cfg_attr(feature = "intrinsics", feature(core_intrinsics))]
#![cfg_attr(doc, doc(include = "../README.md"))]
#![feature(
    allocator_api,
    alloc_layout_extra,
    const_checked_int_methods,
    const_alloc_layout,
    const_fn,
    min_const_generics,
    const_panic,
    const_nonnull_slice_from_raw_parts,
    const_slice_ptr_len,
    nonnull_slice_from_raw_parts,
    slice_ptr_get,
    slice_ptr_len,
    maybe_uninit_slice
)]
#![allow(incomplete_features, clippy::must_use_candidate)]

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

// pub mod stats;

// mod helper;
// #[macro_use]
// mod macros;

// mod affix;
// mod callback_ref;
// mod chunk;
// mod fallback;
mod null;
// mod proxy;
pub mod region;
// mod segregate;

use core::{
    alloc::{AllocError, Layout},
    ptr::NonNull,
};

pub use self::null::Null;

#[cfg(feature = "intrinsics")]
mod intrinsics {
    pub use core::intrinsics::{assume, unlikely};
}

#[cfg(not(feature = "intrinsics"))]
mod intrinsics {
    #![allow(clippy::missing_const_for_fn, clippy::inline_always)]

    #[inline(always)]
    pub fn unlikely(b: bool) -> bool {
        b
    }

    #[inline(always)]
    pub const unsafe fn assume(_: bool) {}
}

#[allow(unused_imports)]
use crate::intrinsics::{assume, unlikely};

#[allow(non_snake_case)]
mod SIZE {}

pub unsafe trait AllocAll {
    /// Attempts to allocate all of the memory the allocator can provide.
    ///
    /// If the allocator is currently not managing any memory, then it returns all the memory
    /// available to the allocator. Subsequent are unlikely to succeed.
    ///
    /// On success, returns `[NonNull<[u8]>]` meeting the size and alignment guarantees of `layout`.
    ///
    /// The returned block may have a larger size than specified by `layout.size()`, and may or may
    /// not have its contents initialized.
    ///
    /// Also see [`AllocRef::alloc`]
    ///
    /// [`AllocRef::alloc`]: core::alloc::AllocRef::alloc
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout` does not meet
    /// allocators size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    fn allocate_all(&self) -> Result<NonNull<[u8]>, AllocError>;

    /// Behaves like `alloc_all`, but also ensures that the returned memory is zero-initialized.
    ///
    /// Also see [`AllocRef::alloc_zeroed`]
    ///
    /// [`AllocRef::alloc_zeroed`]: core::alloc::AllocRef::alloc_zeroed
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout` does not meet
    /// allocators size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    fn allocate_all_zeroed(&self) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.allocate_all()?;
        // SAFETY: `allocate_all` returns a valid memory block
        unsafe { ptr.as_non_null_ptr().as_ptr().write_bytes(0, ptr.len()) }
        Ok(ptr)
    }

    /// Deallocates all the memory the allocator had allocated.
    fn deallocate_all(&self);

    /// Returns the total capacity available in this allocator.
    fn capacity(&self) -> usize;

    /// Returns the free capacity left for allocating.
    fn capacity_left(&self) -> usize;

    /// Returns if the allocator is currently not holding memory.
    fn is_empty(&self) -> bool {
        self.capacity() == self.capacity_left()
    }

    /// Returns if the allocator has no more capacity left.
    fn is_full(&self) -> bool {
        self.capacity_left() == 0
    }
}

pub unsafe trait ReallocateInPlace {
    /// Attempts to extend the memory block.
    ///
    /// Returns the new actual size of the allocated memory. The pointer is suitable for holding
    /// data described by a new layout with `layout`’s alignment and a size given by `new_size`.
    /// To accomplish this, the allocator may extend the allocation referenced by `ptr` to fit the
    /// new layout.
    ///
    /// If this method returns `Err`, the allocator was not able to grow the memory without
    /// changing the pointer. The ownership of the memory block has not been transferred to
    /// this allocator, and the contents of the memory block are unaltered.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator.
    /// * `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
    /// * `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    ///
    /// [*currently allocated*]: https://doc.rust-lang.org/nightly/alloc/alloc/trait.AllocRef.html#currently-allocated-memory
    /// [*fit*]: https://doc.rust-lang.org/nightly/alloc/alloc/trait.AllocRef.html#memory-fitting
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocators size and alignment
    /// constraints of the allocator, or if growing otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    unsafe fn grow_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<usize, AllocError>;

    /// Behaves like `grow_in_place`, but also ensures that the new contents are set to zero before
    /// being returned.
    ///
    /// The memory block will contain the following contents after a successful call to
    /// `grow_zeroed`:
    ///   * Bytes `0..old_layout.size()` are preserved from the original allocation.
    ///   * Bytes `old_layout.size()..old_size` will either be preserved or zeroed, depending on
    ///     the allocator implementation. `old_size` refers to the size of the memory block prior
    ///     to the `grow_zeroed` call, which may be larger than the size that was originally
    ///     requested when it was allocated.
    ///   * Bytes `old_size..new_size` are zeroed. `new_size` refers to the size of the memory
    ///     block returned by the `grow_zeroed` call.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator.
    /// * `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
    /// * `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    ///
    /// [*currently allocated*]: https://doc.rust-lang.org/nightly/alloc/alloc/trait.AllocRef.html#currently-allocated-memory
    /// [*fit*]: https://doc.rust-lang.org/nightly/alloc/alloc/trait.AllocRef.html#memory-fitting
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocators size and alignment
    /// constraints of the allocator, or if growing otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    unsafe fn grow_in_place_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<usize, AllocError>;

    /// Attempts to shrink the memory block.
    ///
    /// Returns the new actual size of the allocated memory. The pointer is suitable for holding
    /// data described by a new layout with `layout`’s alignment and a size given by `new_size`.
    /// To accomplish this, the allocator may extend the allocation referenced by `ptr` to fit the
    /// new layout.
    ///
    /// If this method returns `Err`, the allocator was not able to shrink the memory without
    /// changing the pointer. The ownership of the memory block has not been transferred to
    /// this allocator, and the contents of the memory block are unaltered.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator.
    /// * `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
    /// * `new_layout.size()` must be smaller than or equal to `old_layout.size()`.
    ///
    /// [*currently allocated*]: https://doc.rust-lang.org/nightly/alloc/alloc/trait.AllocRef.html#currently-allocated-memory
    /// [*fit*]: https://doc.rust-lang.org/nightly/alloc/alloc/trait.AllocRef.html#memory-fitting
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and alignment
    /// constraints of the allocator, or if shrinking otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    unsafe fn shrink_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<usize, AllocError>;
}

/// Trait to determine if a given memory block is owned by an allocator.
pub trait Owns {
    /// Returns if the allocator *owns* the passed memory.
    fn owns(&self, ptr: NonNull<[u8]>) -> bool;
}

// #[track_caller]
// #[inline]
// fn check_dealloc_precondition(ptr: NonNull<u8>, old_layout: Layout) {
//     debug_assert!(
//         ptr.as_ptr() as usize >= layout.align(),
//         "`ptr` allocated with the same alignment as `layout.align()`, expected {} >= {}",
//         ptr.as_ptr() as usize,
//         layout.align()
//     );
// }

// #[track_caller]
// #[inline]
// fn check_grow_precondition(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
//     debug_assert!(
//         ptr.as_ptr() as usize >= layout.align(),
//         "`ptr` allocated with the same alignment as `layout.align()`, expected {} >= {}",
//         ptr.as_ptr() as usize,
//         layout.align()
//     );
//     debug_assert!(
//         new_layout >= layout.size(),
//         "`new_layout` must be greater than or equal to `layout.size()`, expected {} >= {}",
//         new_layout,
//         layout.size()
//     );
// }

// #[track_caller]
// #[inline]
// fn check_shrink_precondition(ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
//     debug_assert!(
//         ptr.as_ptr() as usize >= layout.align(),
//         "`ptr` allocated with the same alignment as `layout.align()`, expected {} >= {}",
//         ptr.as_ptr() as usize,
//         layout.align()
//     );
//     debug_assert!(
//         new_layout <= layout.size(),
//         "`new_layout` must be smaller than or equal to `layout.size()`, expected {} <= {}",
//         new_layout,
//         layout.size()
//     );
// }
