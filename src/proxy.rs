use crate::{AllocAll, CallbackRef, Owns, ReallocInPlace};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
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
/// use std::alloc::{AllocRef, Global, Layout};
///
/// let counter = stats::Counter::default();
/// let mut alloc = Proxy {
///     alloc: Global,
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
/// # Ok::<(), core::alloc::AllocErr>(())
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
///     stats::{AllocInitFilter, ResultFilter},
///     Region,
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
/// assert_eq!(counter.num_allocs(), 2);
/// assert_eq!(
///     counter.num_allocs_filter(AllocInitFilter::None, ResultFilter::Ok),
///     1
/// );
/// assert_eq!(
///     counter.num_allocs_filter(AllocInitFilter::Zeroed, ResultFilter::Err),
///     1
/// );
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Proxy<A, C> {
    pub alloc: A,
    pub callbacks: C,
}

unsafe impl<A: AllocRef, C: CallbackRef> AllocRef for Proxy<A, C> {
    #[track_caller]
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.callbacks.before_alloc(layout);
        let result = self.alloc.alloc(layout);
        self.callbacks.after_alloc(layout, result);
        result
    }

    #[track_caller]
    fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.callbacks.before_alloc_zeroed(layout);
        let result = self.alloc.alloc_zeroed(layout);
        self.callbacks.after_alloc_zeroed(layout, result);
        result
    }

    #[track_caller]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        crate::check_dealloc_precondition(ptr, layout);
        self.callbacks.before_dealloc(ptr, layout);
        self.alloc.dealloc(ptr, layout);
        self.callbacks.after_dealloc(ptr, layout);
    }

    #[track_caller]
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        crate::check_grow_precondition(ptr, layout, new_size);
        self.callbacks.before_grow(ptr, layout, new_size);
        let result = self.alloc.grow(ptr, layout, new_size);
        self.callbacks.after_grow(ptr, layout, new_size, result);
        result
    }

    #[track_caller]
    unsafe fn grow_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        crate::check_grow_precondition(ptr, layout, new_size);
        self.callbacks.before_grow_zeroed(ptr, layout, new_size);
        let result = self.alloc.grow_zeroed(ptr, layout, new_size);
        self.callbacks
            .after_grow_zeroed(ptr, layout, new_size, result);
        result
    }

    #[track_caller]
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        crate::check_shrink_precondition(ptr, layout, new_size);
        self.callbacks.before_shrink(ptr, layout, new_size);
        let result = self.alloc.shrink(ptr, layout, new_size);
        self.callbacks.after_shrink(ptr, layout, new_size, result);
        result
    }
}

unsafe impl<A: AllocAll, C: CallbackRef> AllocAll for Proxy<A, C> {
    #[track_caller]
    fn alloc_all(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.callbacks.before_alloc_all(layout);
        let result = self.alloc.alloc_all(layout);
        self.callbacks.after_alloc_all(layout, result);
        result
    }

    #[track_caller]
    fn alloc_all_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.callbacks.before_alloc_all_zeroed(layout);
        let result = self.alloc.alloc_all_zeroed(layout);
        self.callbacks.after_alloc_all_zeroed(layout, result);
        result
    }

    #[track_caller]
    fn dealloc_all(&mut self) {
        self.callbacks.before_dealloc_all();
        self.alloc.dealloc_all();
        self.callbacks.after_dealloc_all();
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

unsafe impl<A: ReallocInPlace, C: CallbackRef> ReallocInPlace for Proxy<A, C> {
    #[track_caller]
    unsafe fn grow_in_place(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, AllocErr> {
        crate::check_grow_precondition(ptr, layout, new_size);
        self.callbacks.before_grow_in_place(ptr, layout, new_size);
        let result = self.alloc.grow_in_place(ptr, layout, new_size);
        self.callbacks
            .after_grow_in_place(ptr, layout, new_size, result);
        result
    }

    #[track_caller]
    unsafe fn grow_in_place_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, AllocErr> {
        crate::check_grow_precondition(ptr, layout, new_size);
        self.callbacks
            .before_grow_in_place_zeroed(ptr, layout, new_size);
        let result = self.alloc.grow_in_place_zeroed(ptr, layout, new_size);
        self.callbacks
            .after_grow_in_place_zeroed(ptr, layout, new_size, result);
        result
    }

    #[track_caller]
    unsafe fn shrink_in_place(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, AllocErr> {
        crate::check_shrink_precondition(ptr, layout, new_size);
        self.callbacks.before_shrink_in_place(ptr, layout, new_size);
        let result = self.alloc.shrink_in_place(ptr, layout, new_size);
        self.callbacks
            .after_shrink_in_place(ptr, layout, new_size, result);
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
