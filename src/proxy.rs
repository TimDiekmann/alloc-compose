use crate::{AllocateAll, CallbackRef, Owns, ReallocateInPlace};
use core::{
    alloc::{AllocError, AllocRef, Layout},
    ptr::NonNull,
};

/// Calls the provided callbacks when invoking methods on `AllocRef`.
///
/// A typical use case for a `Proxy` allocator is collecting statistics. `alloc-compose` provides
/// different implementations for [`CallbackRef`][].
///
/// # Examples
///
/// ```rust
/// #![feature(allocator_api, slice_ptr_get)]
///
/// use alloc_compose::{stats, CallbackRef, Proxy};
/// use std::alloc::{AllocRef, Layout, System};
///
/// let counter = stats::Counter::default();
/// let mut alloc = Proxy {
///     alloc: System,
///     callbacks: counter.by_ref(),
/// };
///
/// unsafe {
///     let memory = alloc.alloc(Layout::new::<u32>())?;
///     alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<u32>());
/// }
///
/// assert_eq!(counter.num_allocs(), 1);
/// assert_eq!(counter.num_deallocs(), 1);
/// # Ok::<(), core::alloc::AllocError>(())
/// ```
///
/// If more information is needed, one can either implement `CallbackRef` itself or use a more
/// fine-grained callback:
///
/// ```rust
/// # #![feature(allocator_api, slice_ptr_get)]
/// # use alloc_compose::{stats, CallbackRef, Proxy};
/// # use std::alloc::{AllocRef, Layout};
/// use alloc_compose::{
///     region::Region,
///     stats::{AllocInitFilter, ResultFilter},
/// };
/// use core::mem::MaybeUninit;
///
/// let counter = stats::FilteredCounter::default();
/// let mut data = [MaybeUninit::new(0); 32];
/// let mut alloc = Proxy {
///     alloc: Region::new(&mut data),
///     callbacks: counter.by_ref(),
/// };
///
/// unsafe {
///     let memory = alloc.alloc(Layout::new::<u32>())?;
///     alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<u32>());
///
///     alloc.alloc_zeroed(Layout::new::<[u32; 64]>()).unwrap_err();
/// }
///
/// assert_eq!(counter.num_allocates(), 2);
/// assert_eq!(
///     counter.num_allocates_filter(AllocInitFilter::None, ResultFilter::Ok),
///     1
/// );
/// assert_eq!(
///     counter.num_allocates_filter(AllocInitFilter::Zeroed, ResultFilter::Err),
///     1
/// );
/// # Ok::<(), core::alloc::AllocError>(())
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Proxy<A, C> {
    pub alloc: A,
    pub callbacks: C,
}

unsafe impl<A: AllocRef, C: CallbackRef> AllocRef for Proxy<A, C> {
    #[track_caller]
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.callbacks.before_allocate(layout);
        let result = self.alloc.alloc(layout);
        self.callbacks.after_allocate(layout, result);
        result
    }

    #[track_caller]
    fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.callbacks.before_allocate_zeroed(layout);
        let result = self.alloc.alloc_zeroed(layout);
        self.callbacks.after_allocate_zeroed(layout, result);
        result
    }

    #[track_caller]
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        crate::check_dealloc_precondition(ptr, layout);
        self.callbacks.before_deallocate(ptr, layout);
        self.alloc.dealloc(ptr, layout);
        self.callbacks.after_deallocate(ptr, layout);
    }

    #[track_caller]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        crate::check_grow_precondition(ptr, old_layout, new_layout);
        self.callbacks.before_grow(ptr, old_layout, new_layout);
        let result = self.alloc.grow(ptr, old_layout, new_layout);
        self.callbacks
            .after_grow(ptr, old_layout, new_layout, result);
        result
    }

    #[track_caller]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        crate::check_grow_precondition(ptr, old_layout, new_layout);
        self.callbacks
            .before_grow_zeroed(ptr, old_layout, new_layout);
        let result = self.alloc.grow_zeroed(ptr, old_layout, new_layout);
        self.callbacks
            .after_grow_zeroed(ptr, old_layout, new_layout, result);
        result
    }

    #[track_caller]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        crate::check_shrink_precondition(ptr, old_layout, new_layout);
        self.callbacks.before_shrink(ptr, old_layout, new_layout);
        let result = self.alloc.shrink(ptr, old_layout, new_layout);
        self.callbacks
            .after_shrink(ptr, old_layout, new_layout, result);
        result
    }
}

unsafe impl<A: AllocateAll, C: CallbackRef> AllocateAll for Proxy<A, C> {
    #[track_caller]
    fn allocate_all(&self) -> Result<NonNull<[u8]>, AllocError> {
        self.callbacks.before_allocate_all();
        let result = self.alloc.allocate_all();
        self.callbacks.after_allocate_all(result);
        result
    }

    #[track_caller]
    fn allocate_all_zeroed(&self) -> Result<NonNull<[u8]>, AllocError> {
        self.callbacks.before_allocate_all_zeroed();
        let result = self.alloc.allocate_all_zeroed();
        self.callbacks.after_allocate_all_zeroed(result);
        result
    }

    #[track_caller]
    fn deallocate_all(&self) {
        self.callbacks.before_deallocate_all();
        self.alloc.deallocate_all();
        self.callbacks.after_deallocate_all();
    }

    #[track_caller]
    #[inline]
    fn capacity(&self) -> usize {
        self.alloc.capacity()
    }

    #[track_caller]
    #[inline]
    fn capacity_left(&self) -> usize {
        self.alloc.capacity_left()
    }

    #[track_caller]
    #[inline]
    fn is_empty(&self) -> bool {
        self.alloc.is_empty()
    }

    #[track_caller]
    #[inline]
    fn is_full(&self) -> bool {
        self.alloc.is_full()
    }
}

unsafe impl<A: ReallocateInPlace, C: CallbackRef> ReallocateInPlace for Proxy<A, C> {
    #[track_caller]
    unsafe fn grow_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<usize, AllocError> {
        crate::check_grow_precondition(ptr, old_layout, new_layout);
        self.callbacks
            .before_grow_in_place(ptr, old_layout, new_layout);
        let result = self.alloc.grow_in_place(ptr, old_layout, new_layout);
        self.callbacks
            .after_grow_in_place(ptr, old_layout, new_layout, result);
        result
    }

    #[track_caller]
    unsafe fn grow_in_place_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<usize, AllocError> {
        crate::check_grow_precondition(ptr, old_layout, new_layout);
        self.callbacks
            .before_grow_in_place_zeroed(ptr, old_layout, new_layout);
        let result = self.alloc.grow_in_place_zeroed(ptr, old_layout, new_layout);
        self.callbacks
            .after_grow_in_place_zeroed(ptr, old_layout, new_layout, result);
        result
    }

    #[track_caller]
    unsafe fn shrink_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<usize, AllocError> {
        crate::check_shrink_precondition(ptr, old_layout, new_layout);
        self.callbacks
            .before_shrink_in_place(ptr, old_layout, new_layout);
        let result = self.alloc.shrink_in_place(ptr, old_layout, new_layout);
        self.callbacks
            .after_shrink_in_place(ptr, old_layout, new_layout, result);
        result
    }
}

impl<A: Owns, C: CallbackRef> Owns for Proxy<A, C> {
    fn owns(&self, ptr: NonNull<[u8]>) -> bool {
        self.callbacks.before_owns();
        let owns = self.alloc.owns(ptr);
        self.callbacks.after_owns(owns);
        owns
    }
}
