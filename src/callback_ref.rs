#[cfg(any(doc, feature = "alloc"))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};
use core::{
    alloc::{AllocErr, AllocInit, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// Backend for the [`Proxy`] allocator.
///
/// As `Callback` is used in `Proxy` and `AllocRef` requires, that a cloned allocator must
/// behave like the same allocator, `Clone` must not be implemented on types, which don't
/// have a shared state. It's possible to use a reference by calling [`by_ref`] or to
/// wrapping them into `Rc` or `Arc` in order to make them cloneable instead. Note, that
/// `Box`, `Rc`, and `Arc` requires the `"alloc"`-feature to be enabled.
///
/// [`by_ref`]: CallbackRef::by_ref
/// [`Proxy`]: crate::Proxy
///
/// # Safety
///
///   * `Clone` must not be implemented on types, which don't have a shared state.
pub unsafe trait CallbackRef {
    /// Called when [`alloc`] was invoked.
    ///
    /// [`alloc`]: core::alloc::AllocRef::alloc
    fn alloc(&self, layout: Layout, init: AllocInit, result: Result<MemoryBlock, AllocErr>);

    /// Called when [`dealloc`] was invoked.
    ///
    /// [`dealloc`]: core::alloc::AllocRef::dealloc
    fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);

    /// Called when [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    fn grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
        result: Result<MemoryBlock, AllocErr>,
    );

    /// Called when [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    fn shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        result: Result<MemoryBlock, AllocErr>,
    );

    /// Called when [`owns`] was invoked.
    ///
    /// [`owns`]: crate::Owns::owns
    fn owns(&self, success: bool);

    /// Creates a "by reference" adaptor for this instance of `CallbackRef`.
    ///
    /// The returned adaptor also implements `CallbackRef` and will simply borrow this.
    #[inline]
    fn by_ref(&self) -> &Self {
        self
    }
}

unsafe impl<C: CallbackRef> CallbackRef for &C {
    #[inline]
    fn alloc(&self, layout: Layout, init: AllocInit, result: Result<MemoryBlock, AllocErr>) {
        (**self).alloc(layout, init, result)
    }

    #[inline]
    fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        (**self).dealloc(ptr, layout)
    }

    #[inline]
    fn grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
        result: Result<MemoryBlock, AllocErr>,
    ) {
        (**self).grow(ptr, layout, new_size, placement, init, result)
    }

    #[inline]
    fn shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        result: Result<MemoryBlock, AllocErr>,
    ) {
        (**self).shrink(ptr, layout, new_size, placement, result)
    }
    #[inline]
    fn owns(&self, success: bool) {
        (**self).owns(success)
    }
}

macro_rules! impl_alloc_stats {
    ($tt:tt) => {
        #[cfg(any(doc, feature = "alloc"))]
        #[cfg_attr(doc, doc(cfg(feature = "alloc")))]
        /// This is only available with the **"alloc"-feature** enabled.
        unsafe impl<C: CallbackRef> CallbackRef for $tt<C> {
            #[inline]
            fn alloc(
                &self,
                layout: Layout,
                init: AllocInit,
                result: Result<MemoryBlock, AllocErr>,
            ) {
                (**self).alloc(layout, init, result)
            }

            #[inline]
            fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
                (**self).dealloc(ptr, layout)
            }

            #[inline]
            fn grow(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                placement: ReallocPlacement,
                init: AllocInit,
                result: Result<MemoryBlock, AllocErr>,
            ) {
                (**self).grow(ptr, layout, new_size, placement, init, result)
            }

            #[inline]
            fn shrink(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                placement: ReallocPlacement,
                result: Result<MemoryBlock, AllocErr>,
            ) {
                (**self).shrink(ptr, layout, new_size, placement, result)
            }

            #[inline]
            fn owns(&self, success: bool) {
                (**self).owns(success)
            }
        }
    };
}

impl_alloc_stats!(Box);
impl_alloc_stats!(Rc);
impl_alloc_stats!(Arc);
