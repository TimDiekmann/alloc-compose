//! Stack-based allocators with user-provided memory
//!
//! A region allocator allocates memory straight from one contiguous chunk. There is no
//! deallocation, and once the region is full, allocation requests returns [`AllocError`].
//! A region only stores a reference to the provided memory and a pointer to the current position.
//!
//! This module provides three kinds of stack-based allocators: [`Region`], [`SharedRegion`], and
//! [`IntrusiveRegion`]. All three allocators uses a user-provided memory to allocate and differ
//! in the way how they store the pointer to the current position.
//!
//! # Which region allocator to chose?
//!
//! Every region allocator is more or less on-par. They slightly differ in performance depending
//! on how many allocations are made and how big the allocations are.
//!
//! - [`Region`] stores a current position in a [`Cell`] right next to the reference to the memory.
//! - [`SharedRegion`] wraps the [`Cell`] in a [`RC`] to support cloning of the allocator.
//! - [`IntrusiveRegion`] stores the pointer to the current position at the end of the provided
//!   memory block.
//!
//! This results in the fact, that [`Region`] cannot be cloned. However, using [`AllocRef::by_ref`]
//! returns a reference to the region, which can itself be cloned.
//! [`SharedRegion`] and [`IntrusiveRegion`] both can be cloned. The [`IntrusiveRegion`] has a
//! better performance in most cases due to cache coherence, but it's hard to say exactly, how much
//! capacity the allocator will have exactly, as the pointer to the current position has to be well
//! aligned. If this feature is important, [`SharedRegion`] or [`Region`] should be used instead.
//! [`SharedRegion`] is only available with the `alloc`-feature, as it requires the [`Rc`] to
//! allocate memory to store the pointer in.
//!
//! [`Rc`]: alloc::rc::Rc
//! [`Cell`]: core::cell::Cell
//!
//! ## Examples
//!
//! ```rust
//! #![feature(allocator_api)]
//!
//! use alloc_compose::{region::Region, Owns};
//! use core::{
//!     alloc::{AllocRef, Layout},
//!     mem::MaybeUninit,
//! };
//!
//! let mut data = [MaybeUninit::uninit(); 64];
//! let region = Region::new(&mut data);
//!
//! let memory = region.alloc(Layout::new::<u32>())?;
//! assert!(region.owns(memory));
//! # Ok::<(), core::alloc::AllocError>(())
//! ```
//!
//! This allocator can also be used in collection types of the std-library:
//!
//! ```rust
//! #![feature(nonnull_slice_from_raw_parts)]
//! # #![feature(allocator_api)]
//! # use alloc_compose::{region::Region, Owns};
//! # use core::{alloc::{AllocRef, Layout}, mem::MaybeUninit};
//! # let mut data = [MaybeUninit::uninit(); 64];
//! # let region = Region::new(&mut data);
//!
//! use core::ptr::NonNull;
//!
//! let mut vec: Vec<u32, _> = Vec::new_in(region.by_ref());
//! vec.extend(&[10, 20, 30]);
//! assert_eq!(vec, [10, 20, 30]);
//!
//! let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };
//! let memory = NonNull::slice_from_raw_parts(ptr.cast(), 12);
//! assert!(region.owns(memory));
//! ```
//!
//! To reset the allocator, [`AllocateAll::deallocate_all`] may be used:
//!
//! ```rust
//! # #![feature(allocator_api)]
//! # use alloc_compose::{region::Region, Owns};
//! # use core::{alloc::{AllocRef, Layout}, mem::MaybeUninit};
//! # let mut data = [MaybeUninit::uninit(); 64];
//! # let region = Region::new(&mut data);
//! # let _ = region.alloc(Layout::new::<u32>())?;
//! use alloc_compose::AllocateAll;
//!
//! assert!(!region.is_empty());
//! region.deallocate_all();
//! assert!(region.is_empty());
//! # Ok::<(), core::alloc::AllocError>(())
//! ```

pub mod raw;

use self::raw::*;
use crate::{AllocateAll, Owns};
use core::{
    alloc::{AllocError, AllocRef, Layout},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
};

/// A stack allocator over an user-defined region of memory.
///
/// It holds a lifetime to the provided memory block, which ensures, that the allocator does not
/// outlive the underlying memory.
///
/// For a version without lifetime see [`RawRegion`] instead.
pub struct Region<'mem> {
    raw: RawRegion,
    _marker: PhantomData<&'mem mut [MaybeUninit<u8>]>,
}

impl<'mem> Region<'mem> {
    /// Creates a new region from the given memory block.
    #[inline]
    pub fn new(memory: &'mem mut [MaybeUninit<u8>]) -> Self {
        let memory = NonNull::from(memory);
        let memory = NonNull::slice_from_raw_parts(memory.cast(), memory.len());
        Self {
            raw: unsafe { RawRegion::new(memory) },
            _marker: PhantomData,
        }
    }
}

/// A clonable region allocator based on `Rc`.
///
/// It holds a lifetime to the provided memory block, which ensures, that the allocator does not
/// outlive the underlying memory.
///
/// For a version without lifetime see [`RawSharedRegion`] instead.
#[derive(Clone)]
#[cfg(any(doc, feature = "alloc"))]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
pub struct SharedRegion<'mem> {
    raw: RawSharedRegion,
    _marker: PhantomData<&'mem mut [MaybeUninit<u8>]>,
}

#[cfg(any(doc, feature = "alloc"))]
impl<'mem> SharedRegion<'mem> {
    /// Creates a new region from the given memory block.
    #[inline]
    pub fn new(memory: &'mem mut [MaybeUninit<u8>]) -> Self {
        let memory = NonNull::from(memory);
        let memory = NonNull::slice_from_raw_parts(memory.cast(), memory.len());
        Self {
            raw: unsafe { RawSharedRegion::new(memory) },
            _marker: PhantomData,
        }
    }
}

/// An intrusive region allocator, which stores the current posision in the provided memory.
///
/// It holds a lifetime to the provided memory block, which ensures, that the allocator does not
/// outlive the underlying memory.
///
/// For a version without lifetime see [`RawIntrusiveRegion`] instead.
#[derive(Clone)]
pub struct IntrusiveRegion<'mem> {
    raw: RawIntrusiveRegion,
    _marker: PhantomData<&'mem mut [MaybeUninit<u8>]>,
}

impl<'mem> IntrusiveRegion<'mem> {
    /// Creates a new region from the given memory block.
    ///
    /// # Panics
    ///
    /// This function panics, when `memory` is not large enough to properly store a pointer.
    #[inline]
    pub fn new(memory: &'mem mut [MaybeUninit<u8>]) -> Self {
        let memory = NonNull::from(memory);
        let memory = NonNull::slice_from_raw_parts(memory.cast(), memory.len());
        Self {
            raw: unsafe { RawIntrusiveRegion::new(memory) },
            _marker: PhantomData,
        }
    }
}

macro_rules! impl_region {
    ($ty:ident, $raw:ty) => {
        impl PartialEq for $ty<'_> {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.raw == rhs.raw
            }
        }

        impl PartialEq<$raw> for $ty<'_> {
            #[inline]
            fn eq(&self, rhs: &$raw) -> bool {
                &self.raw == rhs
            }
        }

        impl PartialEq<$ty<'_>> for $raw {
            #[inline]
            fn eq(&self, rhs: &$ty<'_>) -> bool {
                self == &rhs.raw
            }
        }

        unsafe impl AllocRef for $ty<'_> {
            #[inline]
            fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                self.raw.alloc(layout)
            }

            #[inline]
            unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
                self.raw.dealloc(ptr, layout)
            }

            #[inline]
            unsafe fn grow(
                &self,
                ptr: NonNull<u8>,
                old_layout: Layout,
                new_layout: Layout,
            ) -> Result<NonNull<[u8]>, AllocError> {
                self.raw.grow(ptr, old_layout, new_layout)
            }

            #[inline]
            unsafe fn grow_zeroed(
                &self,
                ptr: NonNull<u8>,
                old_layout: Layout,
                new_layout: Layout,
            ) -> Result<NonNull<[u8]>, AllocError> {
                self.raw.grow(ptr, old_layout, new_layout)
            }

            #[inline]
            unsafe fn shrink(
                &self,
                ptr: NonNull<u8>,
                old_layout: Layout,
                new_layout: Layout,
            ) -> Result<NonNull<[u8]>, AllocError> {
                self.raw.grow(ptr, old_layout, new_layout)
            }
        }

        unsafe impl AllocateAll for $ty<'_> {
            #[inline]
            fn allocate_all(&self) -> Result<NonNull<[u8]>, AllocError> {
                self.raw.allocate_all()
            }

            #[inline]
            fn allocate_all_zeroed(&self) -> Result<NonNull<[u8]>, AllocError> {
                self.raw.allocate_all_zeroed()
            }

            #[inline]
            fn deallocate_all(&self) {
                self.raw.deallocate_all()
            }

            #[inline]
            fn capacity(&self) -> usize {
                self.raw.capacity()
            }

            #[inline]
            fn capacity_left(&self) -> usize {
                self.raw.capacity_left()
            }
        }

        impl Owns for $ty<'_> {
            #[inline]
            fn owns(&self, memory: NonNull<[u8]>) -> bool {
                self.raw.owns(memory)
            }
        }

        impl_global_alloc!($ty<'_>);
    };
}

impl_region!(Region, RawRegion);
#[cfg(any(doc, feature = "alloc"))]
impl_region!(SharedRegion, RawSharedRegion);
impl_region!(IntrusiveRegion, RawIntrusiveRegion);

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]
    use super::*;
    use crate::helper::tracker;
    use core::{cell::Cell, mem};

    fn aligned_slice(memory: &mut [MaybeUninit<u8>], size: usize) -> &mut [MaybeUninit<u8>] {
        let ptr = memory.as_mut_ptr() as usize;
        let start = (ptr + 31) & !(31);
        assert!(memory.len() >= start - ptr + size);
        unsafe { core::slice::from_raw_parts_mut(start as *mut MaybeUninit<u8>, size) }
    }

    macro_rules! impl_tests {
        ($namespace:ident, $ty:ident, $extra:expr) => {
            mod $namespace {
                use super::*;

                #[test]
                fn alloc_zero() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    assert_eq!(region.capacity(), 32);
                    assert!(region.is_empty());

                    region
                        .alloc(Layout::new::<[u8; 0]>())
                        .expect("Could not allocated 0 bytes");
                    assert!(region.is_empty());

                    unsafe {
                        drop(region);
                        assert_eq!(MaybeUninit::slice_assume_init_ref(data)[..32], [1; 32]);
                    }
                }

                #[test]
                fn alloc_zeroed() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    assert_eq!(region.capacity(), 32);
                    assert!(region.is_empty());

                    region
                        .alloc_zeroed(Layout::new::<[u8; 32]>())
                        .expect("Could not allocated 32 bytes");
                    assert!(!region.is_empty());

                    unsafe {
                        drop(region);
                        assert_eq!(MaybeUninit::slice_assume_init_ref(data)[..32], [0; 32]);
                    }
                }

                #[test]
                fn alloc_small() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    assert_eq!(region.capacity(), 32);
                    assert_eq!(region.capacity(), region.capacity_left());

                    region
                        .alloc_zeroed(Layout::new::<[u8; 16]>())
                        .expect("Could not allocated 16 bytes");
                    assert_eq!(region.capacity_left(), 16);

                    unsafe {
                        drop(region);
                        assert_eq!(MaybeUninit::slice_assume_init_ref(&data[0..16]), [1; 16]);
                        assert_eq!(MaybeUninit::slice_assume_init_ref(&data[16..32]), [0; 16]);
                    }
                }

                #[test]
                fn alloc_uninitialzed() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    region
                        .alloc(Layout::new::<[u8; 32]>())
                        .expect("Could not allocated 32 bytes");
                    assert_eq!(region.capacity_left(), 0);

                    unsafe {
                        drop(region);
                        assert_eq!(MaybeUninit::slice_assume_init_ref(&data)[..32], [1; 32]);
                    }
                }

                #[test]
                fn alloc_all() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    assert_eq!(region.capacity(), 32);
                    assert!(region.is_empty());

                    let ptr = region
                        .alloc(Layout::new::<u8>())
                        .expect("Could not allocated 1 byte");
                    assert_eq!(ptr.len(), 1);
                    assert_eq!(region.capacity_left(), 31, "capacity left");

                    let ptr = region
                        .allocate_all_zeroed()
                        .expect("Could not allocated rest of the bytes");
                    assert_eq!(ptr.len(), 31, "len");
                    assert!(region.is_full());

                    region.deallocate_all();
                    assert!(region.is_empty());

                    region
                        .alloc(Layout::new::<[u8; 16]>())
                        .expect("Could not allocate 16 bytes");
                    region
                        .alloc(Layout::new::<[u8; 17]>())
                        .expect_err("Could allocate more than 32 bytes");
                }

                #[test]
                fn alloc_fail() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    region
                        .alloc(Layout::new::<[u8; 33]>())
                        .expect_err("Could allocate 33 bytes");
                }

                #[test]
                fn alloc_aligned() {
                    let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
                    let data = aligned_slice(&mut raw_data, 32 + $extra);
                    let region = tracker(<$ty>::new(data));

                    region
                        .alloc(Layout::from_size_align(5, 1).expect("Invalid layout"))
                        .expect("Could not allocate 5 Bytes");
                    let capacity = region.capacity_left();

                    let ptr = region
                        .alloc(Layout::from_size_align(16, 16).expect("Invalid layout"))
                        .expect("Could not allocate 16 Bytes");
                    assert_eq!(capacity - 16 - 11, region.capacity_left());
                    assert_eq!(ptr.as_mut_ptr() as usize % 16, 0);
                }
            }
        };
    }

    impl_tests!(exclusive, Region, 0);
    #[cfg(any(doc, feature = "alloc"))]
    impl_tests!(shared, SharedRegion, 0);
    impl_tests!(
        intrusive,
        IntrusiveRegion,
        mem::size_of::<NonNull<Cell<NonNull<u8>>>>()
    );

    #[test]
    fn vec() {
        let mut raw_data = [MaybeUninit::<u8>::new(1); 128];
        let data = aligned_slice(&mut raw_data, 32);
        let region = tracker(Region::new(data));
        let mut vec = alloc::vec::Vec::new_in(region.by_ref());
        vec.push(10);
    }

    // #[test]
    // fn dealloc() {
    //     let mut data = [MaybeUninit::new(1); 32];
    //     let mut region = Region::new(&mut data);
    //     let layout = Layout::from_size_align(8, 1).expect("Invalid layout");

    //     let memory = region.alloc(layout).expect("Could not allocate 8 bytes");
    //     assert!(region.owns(memory));
    //     assert_eq!(region.capacity_left(), 24);

    //     unsafe {
    //         region.dealloc(memory.as_non_null_ptr(), layout);
    //     }
    //     assert_eq!(region.capacity_left(), 32);
    //     assert!(!region.owns(memory));

    //     let memory = region.alloc(layout).expect("Could not allocate 8 bytes");
    //     assert!(region.owns(memory));
    //     region.alloc(layout).expect("Could not allocate 8 bytes");
    //     assert!(region.owns(memory));
    //     assert_eq!(memory.len(), 8);
    //     assert_eq!(region.capacity_left(), 16);

    //     unsafe {
    //         region.dealloc(memory.as_non_null_ptr(), layout);
    //     }
    //     // It is not possible to deallocate memory that was not allocated last.
    //     assert!(region.owns(memory));
    //     assert_eq!(region.capacity_left(), 16);
    // }

    // #[test]
    // fn realloc() {
    //     let mut data = [MaybeUninit::new(1); 32];
    //     let mut region = Region::new(&mut data);
    //     let layout = Layout::from_size_align(8, 1).expect("Invalid layout");

    //     let memory = region.alloc(layout).expect("Could not allocate 8 bytes");
    //     assert_eq!(memory.len(), 8);
    //     assert_eq!(region.capacity_left(), 24);

    //     region.alloc(layout).expect("Could not allocate 8 bytes");
    //     assert_eq!(region.capacity_left(), 16);

    //     let memory = unsafe {
    //         region
    //             .grow(memory.as_non_null_ptr(), layout, Layout::new::<[u8; 16]>())
    //             .expect("Could not grow to 16 bytes")
    //     };
    //     assert_eq!(memory.len(), 16);
    //     assert_eq!(region.capacity_left(), 0);

    //     region.dealloc_all();
    //     let memory = region
    //         .alloc_zeroed(Layout::new::<[u8; 16]>())
    //         .expect("Could not allocate 16 bytes");
    //     region
    //         .alloc(Layout::new::<[u8; 8]>())
    //         .expect("Could not allocate 16 bytes");

    //     unsafe {
    //         region
    //             .shrink(
    //                 memory.as_non_null_ptr(),
    //                 Layout::new::<[u8; 16]>(),
    //                 Layout::new::<[u8; 8]>(),
    //             )
    //             .expect("Could not shrink to 8 bytes");
    //     }
    // }

    // #[test]
    // fn debug() {
    //     let test_output = |region: &Region| {
    //         assert_eq!(
    //             format!("{:?}", region),
    //             format!(
    //                 "Region {{ capacity: {}, capacity_left: {} }}",
    //                 region.capacity(),
    //                 region.capacity_left()
    //             )
    //         )
    //     };

    //     let mut data = [MaybeUninit::new(1); 32];
    //     let mut region = Region::new(&mut data);
    //     test_output(&region);

    //     region
    //         .alloc(Layout::new::<[u8; 16]>())
    //         .expect("Could not allocate 16 bytes");
    //     test_output(&region);

    //     region
    //         .alloc(Layout::new::<[u8; 16]>())
    //         .expect("Could not allocate 16 bytes");
    //     test_output(&region);

    //     region.dealloc_all();
    //     test_output(&region);
    // }
}
