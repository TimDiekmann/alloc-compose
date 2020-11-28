//! Region implementations which are not bound by a lifetime.
//!
//! In comparison to the [`region`] module, this module contains the raw counterparts. They don't
//! require a lifetime bound but are `unsafe` to construct, as the user has to ensure, that the
//! allocator outlives the memory.
//!
//! It is highly encouraged to use the safe counterparts whenever possible.
//!
//! [`region`]: crate::region

use crate::{intrinsics::unlikely, AllocateAll, Owns};
use core::{
    alloc::{AllocError, AllocRef, Layout},
    cell::Cell,
    fmt,
    ptr::NonNull,
};

#[cfg(any(doc, feature = "alloc"))]
use alloc::rc::Rc;

trait Current {
    fn current(&self) -> NonNull<u8>;

    #[inline]
    fn current_usize(&self) -> usize {
        self.current().as_ptr() as usize
    }

    fn set_current(&self, ptr: NonNull<u8>);
}

/// A stack allocator over an user-defined region of memory.
///
/// This is the non-lifetime version of [`Region`].
///
/// [`Region`]: crate::region::Region
pub struct RawRegion {
    memory: NonNull<[u8]>,
    current: Cell<NonNull<u8>>,
}

impl RawRegion {
    /// Creates a new region from the given memory block.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `memory` must be [valid] for reads and writes for `memory.len()` many bytes.
    ///
    /// * `memory` must outlive the region.
    ///
    /// * `memory.len()` must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// For a safe variant use [`Region`] instead.
    ///
    /// [`Region`]: crate::region::Region
    /// [valid]: core::ptr#safety
    /// [`pointer::offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
    #[inline]
    pub unsafe fn new(memory: NonNull<[u8]>) -> Self {
        Self {
            memory,
            current: Cell::new(end(memory)),
        }
    }
}

impl Current for RawRegion {
    #[inline]
    fn current(&self) -> NonNull<u8> {
        self.current.get()
    }

    #[inline]
    fn set_current(&self, ptr: NonNull<u8>) {
        self.current.set(ptr)
    }
}

#[derive(Clone)]
#[cfg(any(doc, feature = "alloc"))]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
pub struct RawSharedRegion {
    memory: NonNull<[u8]>,
    current: Rc<Cell<NonNull<u8>>>,
}

/// A clonable region allocator based on `Rc`.
///
/// This is the non-lifetime version of [`SharedRegion`].
///
/// [`SharedRegion`]: crate::region::SharedRegion
#[cfg(any(doc, feature = "alloc"))]
impl RawSharedRegion {
    /// Creates a new region from the given memory block.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `memory` must be [valid] for reads and writes for `memory.len()` many bytes.
    ///
    /// * `memory` must outlive the region.
    ///
    /// * `memory.len()` must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// For a safe variant use [`SharedRegion`] instead.
    ///
    /// [`SharedRegion`]: crate::region::SharedRegion
    /// [valid]: core::ptr#safety
    /// [`pointer::offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
    #[inline]
    pub unsafe fn new(memory: NonNull<[u8]>) -> Self {
        Self {
            memory,
            current: Rc::new(Cell::new(end(memory))),
        }
    }
}

#[cfg(any(doc, feature = "alloc"))]
impl Current for RawSharedRegion {
    #[inline]
    fn current(&self) -> NonNull<u8> {
        self.current.get()
    }

    #[inline]
    fn set_current(&self, ptr: NonNull<u8>) {
        self.current.set(ptr)
    }
}

/// An intrusive region allocator, which stores the current posision in the provided memory.
///
/// This is the non-lifetime version of [`IntrusiveRegion`].
///
/// [`IntrusiveRegion`]: crate::region::IntrusiveRegion
#[derive(Clone)]
pub struct RawIntrusiveRegion {
    memory: NonNull<[u8]>,
    current: NonNull<Cell<NonNull<u8>>>,
}

impl RawIntrusiveRegion {
    /// Creates a new region from the given memory block.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `memory` must be [valid] for reads and writes for `memory.len()` many bytes.
    ///
    /// * `memory` must outlive the region.
    ///
    /// * `memory.len()` must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// For a safe variant use [`IntrusiveRegion`] instead.
    ///
    /// [`IntrusiveRegion`]: crate::region::IntrusiveRegion
    /// [valid]: core::ptr#safety
    /// [`pointer::offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
    ///
    /// # Panics
    ///
    /// This function panics, when `memory` is not large enough to properly store a pointer.
    #[inline]
    pub unsafe fn new(memory: NonNull<[u8]>) -> Self {
        let current: NonNull<Cell<NonNull<u8>>> = alloc_impl(
            memory,
            end(memory),
            Layout::new::<NonNull<Cell<NonNull<u8>>>>(),
        )
        .expect("Could not store pointer in region")
        .as_non_null_ptr()
        .cast();
        current.as_ptr().write(Cell::new(current.cast()));
        let memory = NonNull::slice_from_raw_parts(
            memory.as_non_null_ptr(),
            current.as_ptr() as usize - memory.as_mut_ptr() as usize,
        );
        Self { memory, current }
    }
}

impl Current for RawIntrusiveRegion {
    #[inline]
    fn current(&self) -> NonNull<u8> {
        unsafe { self.current.as_ref().get() }
    }

    #[inline]
    fn set_current(&self, ptr: NonNull<u8>) {
        unsafe { self.current.as_ref().set(ptr) }
    }
}

#[inline]
fn alloc_impl(
    memory: NonNull<[u8]>,
    current: NonNull<u8>,
    layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    let current = current.as_ptr() as usize;
    let new = current.checked_sub(layout.size()).ok_or(AllocError)?;
    let aligned = (new & !(layout.align() - 1)) as *mut u8;

    if unlikely(aligned < memory.as_mut_ptr()) {
        Err(AllocError)
    } else {
        Ok(NonNull::slice_from_raw_parts(
            unsafe { NonNull::new_unchecked(aligned) },
            current - aligned as usize,
        ))
    }
}

#[inline]
fn alloc_all_impl(
    memory: NonNull<[u8]>,
    current: NonNull<u8>,
) -> Result<NonNull<[u8]>, AllocError> {
    let current = current.as_ptr() as usize;
    let new = memory.as_non_null_ptr();

    Ok(NonNull::slice_from_raw_parts(
        new,
        current - new.as_ptr() as usize,
    ))
}

#[inline]
fn end(ptr: NonNull<[u8]>) -> NonNull<u8> {
    unsafe { NonNull::new_unchecked(ptr.as_mut_ptr().add(ptr.len())) }
}

// unsafe impl AllocRef for RawRegion {
//     #[inline]
//     fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         let new = alloc_impl(self.memory, self.current.get(), layout)?;
//         self.current.set(new.as_non_null_ptr());
//         Ok(new)
//     }

//     #[inline]
//     unsafe fn dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {}
// }

// unsafe impl AllocRef for RawSharedRegion {
//     #[inline]
//     fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         let current = self.current.as_ref();
//         let new = alloc_impl(self.memory, current.get(), layout)?;
//         current.set(new.as_non_null_ptr());
//         Ok(new)
//     }

//     #[inline]
//     unsafe fn dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {}
// }

// unsafe impl AllocRef for RawIntrusiveRegion {
//     #[inline]
//     fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         let current = unsafe { self.current.as_ref() };
//         let new = alloc_impl(self.memory, current.get(), layout)?;
//         current.set(new.as_non_null_ptr());
//         Ok(new)
//     }

//     #[inline]
//     unsafe fn dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {}
// }

macro_rules! impl_raw_region {
    ($ty:ident) => {
        impl PartialEq for $ty {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.memory == rhs.memory
            }
        }

        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("RawRegion")
                    .field("memory", &self.memory)
                    .field("len", &self.memory.len())
                    .field("current", &self.current())
                    .finish()
            }
        }

        unsafe impl AllocRef for $ty {
            #[inline]
            fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                let new = alloc_impl(self.memory, self.current(), layout)?;
                self.set_current(new.as_non_null_ptr());
                Ok(new)
            }

            #[inline]
            unsafe fn dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {}

            unsafe fn grow(
                &self,
                ptr: NonNull<u8>,
                old_layout: Layout,
                new_layout: Layout,
            ) -> Result<NonNull<[u8]>, AllocError> {
                Err(AllocError)
            }

            unsafe fn grow_zeroed(
                &self,
                ptr: NonNull<u8>,
                old_layout: Layout,
                new_layout: Layout,
            ) -> Result<NonNull<[u8]>, AllocError> {
                Err(AllocError)
            }

            unsafe fn shrink(
                &self,
                ptr: NonNull<u8>,
                old_layout: Layout,
                new_layout: Layout,
            ) -> Result<NonNull<[u8]>, AllocError> {
                Err(AllocError)
            }
        }

        unsafe impl AllocateAll for $ty {
            #[inline]
            fn allocate_all(&self) -> Result<NonNull<[u8]>, AllocError> {
                let new = alloc_all_impl(self.memory, self.current())?;
                self.set_current(new.as_non_null_ptr());
                Ok(new)
            }

            #[inline]
            fn deallocate_all(&self) {
                self.set_current(end(self.memory))
            }

            #[inline]
            fn capacity(&self) -> usize {
                self.memory.len()
            }

            #[inline]
            fn capacity_left(&self) -> usize {
                self.current_usize() - self.memory.as_mut_ptr() as usize
            }
        }

        impl Owns for $ty {
            #[inline]
            fn owns(&self, memory: NonNull<[u8]>) -> bool {
                let ptr = memory.as_mut_ptr() as usize;
                let current = self.current_usize();
                ptr >= current && ptr + memory.len() <= end(self.memory).as_ptr() as usize
            }
        }

        impl_global_alloc!($ty);
    };
}

impl_raw_region!(RawRegion);
#[cfg(any(doc, feature = "alloc"))]
impl_raw_region!(RawSharedRegion);
impl_raw_region!(RawIntrusiveRegion);
