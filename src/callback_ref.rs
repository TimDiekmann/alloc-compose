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
///   * `Clone` must not be implemented on types, which don't have a shared state.
#[allow(unused_variables)]
pub unsafe trait CallbackRef {
    /// Called before [`alloc`] was invoked.
    ///
    /// [`alloc`]: core::alloc::AllocRef::alloc
    #[inline]
    fn before_alloc(&self, layout: Layout, init: AllocInit) {}

    /// Called after [`alloc`] was invoked.
    ///
    /// [`alloc`]: core::alloc::AllocRef::alloc
    #[inline]
    fn after_alloc(&self, layout: Layout, init: AllocInit, result: Result<MemoryBlock, AllocErr>) {}

    /// Called before [`dealloc`] was invoked.
    ///
    /// [`dealloc`]: core::alloc::AllocRef::dealloc
    #[inline]
    fn before_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {}

    /// Called after [`dealloc`] was invoked.
    ///
    /// [`dealloc`]: core::alloc::AllocRef::dealloc
    #[inline]
    fn after_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {}

    /// Called before [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    #[inline]
    fn before_grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) {
    }

    /// Called after [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    #[inline]
    fn after_grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
        result: Result<MemoryBlock, AllocErr>,
    ) {
    }

    /// Called before [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    #[inline]
    fn before_shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) {
    }

    /// Called after [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    #[inline]
    fn after_shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        result: Result<MemoryBlock, AllocErr>,
    ) {
    }

    /// Called before [`owns`] was invoked.
    ///
    /// [`owns`]: crate::Owns::owns
    #[inline]
    fn before_owns(&self) {}

    /// Called after [`owns`] was invoked.
    ///
    /// [`owns`]: crate::Owns::owns
    #[inline]
    fn after_owns(&self, success: bool) {}

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
    fn before_alloc(&self, layout: Layout, init: AllocInit) {
        (**self).before_alloc(layout, init)
    }

    #[inline]
    fn after_alloc(&self, layout: Layout, init: AllocInit, result: Result<MemoryBlock, AllocErr>) {
        (**self).after_alloc(layout, init, result)
    }

    #[inline]
    fn before_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        (**self).before_dealloc(ptr, layout)
    }

    #[inline]
    fn after_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        (**self).after_dealloc(ptr, layout)
    }

    #[inline]
    fn before_shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) {
        (**self).before_shrink(ptr, layout, new_size, placement)
    }

    #[inline]
    fn after_shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        result: Result<MemoryBlock, AllocErr>,
    ) {
        (**self).after_shrink(ptr, layout, new_size, placement, result)
    }

    #[inline]
    fn before_grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) {
        (**self).before_grow(ptr, layout, new_size, placement, init)
    }

    #[inline]
    fn after_grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
        result: Result<MemoryBlock, AllocErr>,
    ) {
        (**self).after_grow(ptr, layout, new_size, placement, init, result)
    }

    #[inline]
    fn before_owns(&self) {
        (**self).before_owns()
    }

    #[inline]
    fn after_owns(&self, success: bool) {
        (**self).after_owns(success)
    }
}

macro_rules! impl_alloc_stats {
    ($tt:tt) => {
        #[cfg(any(doc, feature = "alloc"))]
        #[cfg_attr(doc, doc(cfg(feature = "alloc")))]
        /// This is only available with the **"alloc"-feature** enabled.
        unsafe impl<C: CallbackRef> CallbackRef for $tt<C> {
            #[inline]
            fn before_alloc(&self, layout: Layout, init: AllocInit) {
                (**self).before_alloc(layout, init)
            }

            #[inline]
            fn after_alloc(
                &self,
                layout: Layout,
                init: AllocInit,
                result: Result<MemoryBlock, AllocErr>,
            ) {
                (**self).after_alloc(layout, init, result)
            }

            #[inline]
            fn before_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
                (**self).before_dealloc(ptr, layout)
            }

            #[inline]
            fn after_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
                (**self).after_dealloc(ptr, layout)
            }

            #[inline]
            fn before_shrink(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                placement: ReallocPlacement,
            ) {
                (**self).before_shrink(ptr, layout, new_size, placement)
            }

            #[inline]
            fn after_shrink(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                placement: ReallocPlacement,
                result: Result<MemoryBlock, AllocErr>,
            ) {
                (**self).after_shrink(ptr, layout, new_size, placement, result)
            }

            #[inline]
            fn before_grow(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                placement: ReallocPlacement,
                init: AllocInit,
            ) {
                (**self).before_grow(ptr, layout, new_size, placement, init)
            }

            #[inline]
            fn after_grow(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                placement: ReallocPlacement,
                init: AllocInit,
                result: Result<MemoryBlock, AllocErr>,
            ) {
                (**self).after_grow(ptr, layout, new_size, placement, init, result)
            }

            #[inline]
            fn before_owns(&self) {
                (**self).before_owns()
            }

            #[inline]
            fn after_owns(&self, success: bool) {
                (**self).after_owns(success)
            }
        }
    };
}

impl_alloc_stats!(Box);
impl_alloc_stats!(Rc);
impl_alloc_stats!(Arc);

#[cfg(test)]
mod tests {
    use crate::CallbackRef;
    use std::{
        alloc::{AllocErr, AllocInit, Layout, MemoryBlock, ReallocPlacement},
        cell::Cell,
        ptr::NonNull,
        rc::Rc,
        sync::Arc,
    };

    #[derive(Default)]
    struct Callback {
        before_alloc: Cell<u32>,
        after_alloc: Cell<u32>,
        before_dealloc: Cell<u32>,
        after_dealloc: Cell<u32>,
        before_shrink: Cell<u32>,
        after_shrink: Cell<u32>,
        before_grow: Cell<u32>,
        after_grow: Cell<u32>,
        before_owns: Cell<u32>,
        after_owns: Cell<u32>,
    }

    unsafe impl CallbackRef for Callback {
        fn before_alloc(&self, _layout: Layout, _init: AllocInit) {
            self.before_alloc.set(self.before_alloc.get() + 1)
        }
        fn after_alloc(
            &self,
            _layout: Layout,
            _init: AllocInit,
            _result: Result<MemoryBlock, AllocErr>,
        ) {
            self.after_alloc.set(self.after_alloc.get() + 1)
        }
        fn before_dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {
            self.before_dealloc.set(self.before_dealloc.get() + 1)
        }
        fn after_dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {
            self.after_dealloc.set(self.after_dealloc.get() + 1)
        }
        fn before_grow(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _placement: ReallocPlacement,
            _init: AllocInit,
        ) {
            self.before_grow.set(self.before_grow.get() + 1)
        }
        fn after_grow(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _placement: ReallocPlacement,
            _init: AllocInit,
            _result: Result<MemoryBlock, AllocErr>,
        ) {
            self.after_grow.set(self.after_grow.get() + 1)
        }
        fn before_shrink(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _placement: ReallocPlacement,
        ) {
            self.before_shrink.set(self.before_shrink.get() + 1)
        }
        fn after_shrink(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _placement: ReallocPlacement,
            _result: Result<MemoryBlock, AllocErr>,
        ) {
            self.after_shrink.set(self.after_shrink.get() + 1)
        }
        fn before_owns(&self) {
            self.before_owns.set(self.before_owns.get() + 1)
        }
        fn after_owns(&self, _success: bool) {
            self.after_owns.set(self.after_owns.get() + 1)
        }
    }

    fn test_callback(callback: impl CallbackRef) {
        callback.before_alloc(Layout::new::<()>(), AllocInit::Uninitialized);
        callback.after_alloc(Layout::new::<()>(), AllocInit::Uninitialized, Err(AllocErr));
        callback.before_dealloc(NonNull::dangling(), Layout::new::<()>());
        callback.after_dealloc(NonNull::dangling(), Layout::new::<()>());
        callback.before_grow(
            NonNull::dangling(),
            Layout::new::<()>(),
            0,
            ReallocPlacement::MayMove,
            AllocInit::Uninitialized,
        );
        callback.after_grow(
            NonNull::dangling(),
            Layout::new::<()>(),
            0,
            ReallocPlacement::MayMove,
            AllocInit::Uninitialized,
            Err(AllocErr),
        );
        callback.before_shrink(
            NonNull::dangling(),
            Layout::new::<()>(),
            0,
            ReallocPlacement::MayMove,
        );
        callback.after_shrink(
            NonNull::dangling(),
            Layout::new::<()>(),
            0,
            ReallocPlacement::MayMove,
            Err(AllocErr),
        );
        callback.before_owns();
        callback.after_owns(false);
    }

    fn check_counts(callback: &Callback) {
        assert_eq!(callback.before_alloc.get(), 1);
        assert_eq!(callback.after_alloc.get(), 1);
        assert_eq!(callback.before_dealloc.get(), 1);
        assert_eq!(callback.after_dealloc.get(), 1);
        assert_eq!(callback.before_grow.get(), 1);
        assert_eq!(callback.after_grow.get(), 1);
        assert_eq!(callback.before_shrink.get(), 1);
        assert_eq!(callback.after_shrink.get(), 1);
        assert_eq!(callback.before_owns.get(), 1);
        assert_eq!(callback.after_owns.get(), 1);
    }

    #[test]
    fn plain() {
        let callback = Callback::default();
        test_callback(callback.by_ref());
        check_counts(&callback);
    }

    #[test]
    fn boxed() {
        let callback = Box::new(Callback::default());
        test_callback(callback.by_ref());
        check_counts(&callback);
    }

    #[test]
    fn rc() {
        let callback = Rc::new(Callback::default());
        test_callback(callback.by_ref());
        check_counts(&callback);
    }

    #[test]
    fn arc() {
        let callback = Arc::new(Callback::default());
        test_callback(callback.by_ref());
        check_counts(&callback);
    }
}
