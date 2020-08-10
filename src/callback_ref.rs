#[cfg(any(doc, feature = "alloc"))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};
use core::{
    alloc::{AllocErr, Layout},
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
    fn before_alloc(&self, layout: Layout) {}

    /// Called after [`alloc`] was invoked.
    ///
    /// [`alloc`]: core::alloc::AllocRef::alloc
    #[inline]
    fn after_alloc(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {}

    /// Called before [`alloc_zeroed`] was invoked.
    ///
    /// [`alloc_zeroed`]: core::alloc::AllocRef::alloc_zeroed
    #[inline]
    fn before_alloc_zeroed(&self, layout: Layout) {}

    /// Called after [`alloc_zeroed`] was invoked.
    ///
    /// [`alloc_zeroed`]: core::alloc::AllocRef::alloc_zeroed
    #[inline]
    fn after_alloc_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {}

    /// Called before [`alloc_all`] was invoked.
    ///
    /// [`alloc_all`]: crate::AllocAll::alloc_all
    #[inline]
    fn before_alloc_all(&self, layout: Layout) {}

    /// Called after [`alloc_all`] was invoked.
    ///
    /// [`alloc_all`]: crate::AllocAll::alloc_all
    #[inline]
    fn after_alloc_all(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {}

    /// Called before [`alloc_all_zeroed`] was invoked.
    ///
    /// [`alloc_all_zeroed`]: crate::AllocAll::alloc_all_zeroed
    #[inline]
    fn before_alloc_all_zeroed(&self, layout: Layout) {}

    /// Called after [`alloc_all_zeroed`] was invoked.
    ///
    /// [`alloc_all_zeroed`]: crate::AllocAll::alloc_all_zeroed
    #[inline]
    fn after_alloc_all_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {}

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

    /// Called before [`dealloc_all`] was invoked.
    ///
    /// [`dealloc_all`]: crate::AllocAll::dealloc_all
    #[inline]
    fn before_dealloc_all(&self) {}

    /// Called after [`dealloc_all`] was invoked.
    ///
    /// [`dealloc_all`]: crate::AllocAll::dealloc_all
    #[inline]
    fn after_dealloc_all(&self) {}

    /// Called before [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    #[inline]
    fn before_grow(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {}

    /// Called after [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    #[inline]
    fn after_grow(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        result: Result<NonNull<[u8]>, AllocErr>,
    ) {
    }

    /// Called before [`grow_zeroed`] was invoked.
    ///
    /// [`grow_zeroed`]: core::alloc::AllocRef::grow_zeroed
    #[inline]
    fn before_grow_zeroed(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {}

    /// Called after [`grow_zeroed`] was invoked.
    ///
    /// [`grow_zeroed`]: core::alloc::AllocRef::grow_zeroed
    #[inline]
    fn after_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        result: Result<NonNull<[u8]>, AllocErr>,
    ) {
    }

    /// Called before [`grow_in_place`] was invoked.
    ///
    /// [`grow_in_place`]: crate::ReallocInPlace::grow_in_place
    #[inline]
    fn before_grow_in_place(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {}

    /// Called after [`grow_in_place`] was invoked.
    ///
    /// [`grow_in_place`]: crate::ReallocInPlace::grow_in_place
    #[inline]
    fn after_grow_in_place(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        result: Result<usize, AllocErr>,
    ) {
    }

    /// Called before [`grow_in_place_zeroed`] was invoked.
    ///
    /// [`grow_in_place_zeroed`]: crate::ReallocInPlace::grow_in_place_zeroed
    #[inline]
    fn before_grow_in_place_zeroed(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {}

    /// Called after [`grow_in_place_zeroed`] was invoked.
    ///
    /// [`grow_in_place_zeroed`]: crate::ReallocInPlace::grow_in_place_zeroed
    #[inline]
    fn after_grow_in_place_zeroed(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        result: Result<usize, AllocErr>,
    ) {
    }

    /// Called before [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    #[inline]
    fn before_shrink(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {}

    /// Called after [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    #[inline]
    fn after_shrink(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        result: Result<NonNull<[u8]>, AllocErr>,
    ) {
    }

    /// Called before [`shrink_in_place`] was invoked.
    ///
    /// [`shrink_in_place`]: crate::ReallocInPlace::shrink_in_place
    #[inline]
    fn before_shrink_in_place(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {}

    /// Called after [`shrink_in_place`] was invoked.
    ///
    /// [`shrink_in_place`]: crate::ReallocInPlace::shrink_in_place
    #[inline]
    fn after_shrink_in_place(
        &self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        result: Result<usize, AllocErr>,
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

macro_rules! impl_alloc_stats {
    ($ty:ty) => {
        unsafe impl<C: CallbackRef> CallbackRef for $ty {
            #[inline]
            fn before_alloc(&self, layout: Layout) {
                (**self).before_alloc(layout)
            }

            #[inline]
            fn after_alloc(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
                (**self).after_alloc(layout, result)
            }

            #[inline]
            fn before_alloc_zeroed(&self, layout: Layout) {
                (**self).before_alloc_zeroed(layout)
            }

            #[inline]
            fn after_alloc_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
                (**self).after_alloc_zeroed(layout, result)
            }

            #[inline]
            fn before_alloc_all(&self, layout: Layout) {
                (**self).before_alloc_all(layout)
            }

            #[inline]
            fn after_alloc_all(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
                (**self).after_alloc_all(layout, result)
            }

            #[inline]
            fn before_alloc_all_zeroed(&self, layout: Layout) {
                (**self).before_alloc_all_zeroed(layout)
            }

            #[inline]
            fn after_alloc_all_zeroed(
                &self,
                layout: Layout,
                result: Result<NonNull<[u8]>, AllocErr>,
            ) {
                (**self).after_alloc_all_zeroed(layout, result)
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
            fn before_dealloc_all(&self) {
                (**self).before_dealloc_all()
            }

            #[inline]
            fn after_dealloc_all(&self) {
                (**self).after_dealloc_all()
            }

            #[inline]
            fn before_grow(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
                (**self).before_grow(ptr, layout, new_size)
            }

            #[inline]
            fn after_grow(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                result: Result<NonNull<[u8]>, AllocErr>,
            ) {
                (**self).after_grow(ptr, layout, new_size, result)
            }

            #[inline]
            fn before_grow_zeroed(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
                (**self).before_grow_zeroed(ptr, layout, new_size)
            }

            #[inline]
            fn after_grow_zeroed(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                result: Result<NonNull<[u8]>, AllocErr>,
            ) {
                (**self).after_grow_zeroed(ptr, layout, new_size, result)
            }

            #[inline]
            fn before_grow_in_place(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
                (**self).before_grow_in_place(ptr, layout, new_size)
            }

            #[inline]
            fn after_grow_in_place(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                result: Result<usize, AllocErr>,
            ) {
                (**self).after_grow_in_place(ptr, layout, new_size, result)
            }

            #[inline]
            fn before_grow_in_place_zeroed(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
            ) {
                (**self).before_grow_in_place_zeroed(ptr, layout, new_size)
            }

            #[inline]
            fn after_grow_in_place_zeroed(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                result: Result<usize, AllocErr>,
            ) {
                (**self).after_grow_in_place_zeroed(ptr, layout, new_size, result)
            }

            #[inline]
            fn before_shrink(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
                (**self).before_shrink(ptr, layout, new_size)
            }

            #[inline]
            fn after_shrink(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                result: Result<NonNull<[u8]>, AllocErr>,
            ) {
                (**self).after_shrink(ptr, layout, new_size, result)
            }

            #[inline]
            fn before_shrink_in_place(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
                (**self).before_shrink_in_place(ptr, layout, new_size)
            }

            #[inline]
            fn after_shrink_in_place(
                &self,
                ptr: NonNull<u8>,
                layout: Layout,
                new_size: usize,
                result: Result<usize, AllocErr>,
            ) {
                (**self).after_shrink_in_place(ptr, layout, new_size, result)
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

impl_alloc_stats!(&C);
#[cfg(any(doc, feature = "alloc"))]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl_alloc_stats!(Box<C>);
#[cfg(any(doc, feature = "alloc"))]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl_alloc_stats!(Rc<C>);
#[cfg(any(doc, feature = "alloc"))]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl_alloc_stats!(Arc<C>);

#[cfg(test)]
mod tests {
    use crate::CallbackRef;
    use std::{
        alloc::{AllocErr, Layout},
        cell::Cell,
        ptr::NonNull,
        rc::Rc,
        sync::Arc,
    };

    #[derive(Default)]
    struct Callback {
        before_alloc: Cell<u32>,
        after_alloc: Cell<u32>,
        before_alloc_zeroed: Cell<u32>,
        after_alloc_zeroed: Cell<u32>,
        before_alloc_all: Cell<u32>,
        after_alloc_all: Cell<u32>,
        before_alloc_all_zeroed: Cell<u32>,
        after_alloc_all_zeroed: Cell<u32>,
        before_dealloc: Cell<u32>,
        after_dealloc: Cell<u32>,
        before_dealloc_all: Cell<u32>,
        after_dealloc_all: Cell<u32>,
        before_grow: Cell<u32>,
        after_grow: Cell<u32>,
        before_grow_zeroed: Cell<u32>,
        after_grow_zeroed: Cell<u32>,
        before_grow_in_place: Cell<u32>,
        after_grow_in_place: Cell<u32>,
        before_grow_in_place_zeroed: Cell<u32>,
        after_grow_in_place_zeroed: Cell<u32>,
        before_shrink: Cell<u32>,
        after_shrink: Cell<u32>,
        before_shrink_in_place: Cell<u32>,
        after_shrink_in_place: Cell<u32>,
        before_owns: Cell<u32>,
        after_owns: Cell<u32>,
    }

    unsafe impl CallbackRef for Callback {
        fn before_alloc(&self, _layout: Layout) {
            self.before_alloc.set(self.before_alloc.get() + 1)
        }
        fn after_alloc(&self, _layout: Layout, _result: Result<NonNull<[u8]>, AllocErr>) {
            self.after_alloc.set(self.after_alloc.get() + 1)
        }
        fn before_alloc_zeroed(&self, _layout: Layout) {
            self.before_alloc_zeroed
                .set(self.before_alloc_zeroed.get() + 1)
        }
        fn after_alloc_zeroed(&self, _layout: Layout, _result: Result<NonNull<[u8]>, AllocErr>) {
            self.after_alloc_zeroed
                .set(self.after_alloc_zeroed.get() + 1)
        }
        fn before_alloc_all(&self, _layout: Layout) {
            self.before_alloc_all.set(self.before_alloc_all.get() + 1)
        }
        fn after_alloc_all(&self, _layout: Layout, _result: Result<NonNull<[u8]>, AllocErr>) {
            self.after_alloc_all.set(self.after_alloc_all.get() + 1)
        }
        fn before_alloc_all_zeroed(&self, _layout: Layout) {
            self.before_alloc_all_zeroed
                .set(self.before_alloc_all_zeroed.get() + 1)
        }
        fn after_alloc_all_zeroed(
            &self,
            _layout: Layout,
            _result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            self.after_alloc_all_zeroed
                .set(self.after_alloc_all_zeroed.get() + 1)
        }
        fn before_dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {
            self.before_dealloc.set(self.before_dealloc.get() + 1)
        }
        fn after_dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {
            self.after_dealloc.set(self.after_dealloc.get() + 1)
        }
        fn before_dealloc_all(&self) {
            self.before_dealloc_all
                .set(self.before_dealloc_all.get() + 1)
        }
        fn after_dealloc_all(&self) {
            self.after_dealloc_all.set(self.after_dealloc_all.get() + 1)
        }
        fn before_grow(&self, _ptr: NonNull<u8>, _layout: Layout, _new_size: usize) {
            self.before_grow.set(self.before_grow.get() + 1)
        }
        fn after_grow(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            self.after_grow.set(self.after_grow.get() + 1)
        }
        fn before_grow_zeroed(&self, _ptr: NonNull<u8>, _layout: Layout, _new_size: usize) {
            self.before_grow_zeroed
                .set(self.before_grow_zeroed.get() + 1)
        }
        fn after_grow_zeroed(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            self.after_grow_zeroed.set(self.after_grow_zeroed.get() + 1)
        }
        fn before_grow_in_place(&self, _ptr: NonNull<u8>, _layout: Layout, _new_size: usize) {
            self.before_grow_in_place
                .set(self.before_grow_in_place.get() + 1)
        }
        fn after_grow_in_place(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _result: Result<usize, AllocErr>,
        ) {
            self.after_grow_in_place
                .set(self.after_grow_in_place.get() + 1)
        }
        fn before_grow_in_place_zeroed(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
        ) {
            self.before_grow_in_place_zeroed
                .set(self.before_grow_in_place_zeroed.get() + 1)
        }
        fn after_grow_in_place_zeroed(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _result: Result<usize, AllocErr>,
        ) {
            self.after_grow_in_place_zeroed
                .set(self.after_grow_in_place_zeroed.get() + 1)
        }
        fn before_shrink(&self, _ptr: NonNull<u8>, _layout: Layout, _new_size: usize) {
            self.before_shrink.set(self.before_shrink.get() + 1)
        }
        fn after_shrink(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            self.after_shrink.set(self.after_shrink.get() + 1)
        }
        fn before_shrink_in_place(&self, _ptr: NonNull<u8>, _layout: Layout, _new_size: usize) {
            self.before_shrink_in_place
                .set(self.before_shrink_in_place.get() + 1)
        }
        fn after_shrink_in_place(
            &self,
            _ptr: NonNull<u8>,
            _layout: Layout,
            _new_size: usize,
            _result: Result<usize, AllocErr>,
        ) {
            self.after_shrink_in_place
                .set(self.after_shrink_in_place.get() + 1)
        }
        fn before_owns(&self) {
            self.before_owns.set(self.before_owns.get() + 1)
        }
        fn after_owns(&self, _success: bool) {
            self.after_owns.set(self.after_owns.get() + 1)
        }
    }

    fn test_callback(callback: impl CallbackRef) {
        callback.before_alloc(Layout::new::<()>());
        callback.after_alloc(Layout::new::<()>(), Err(AllocErr));
        callback.before_alloc_zeroed(Layout::new::<()>());
        callback.after_alloc_zeroed(Layout::new::<()>(), Err(AllocErr));
        callback.before_alloc_all(Layout::new::<()>());
        callback.after_alloc_all(Layout::new::<()>(), Err(AllocErr));
        callback.before_alloc_all_zeroed(Layout::new::<()>());
        callback.after_alloc_all_zeroed(Layout::new::<()>(), Err(AllocErr));
        callback.before_dealloc(NonNull::dangling(), Layout::new::<()>());
        callback.after_dealloc(NonNull::dangling(), Layout::new::<()>());
        callback.before_dealloc_all();
        callback.after_dealloc_all();
        callback.before_grow(NonNull::dangling(), Layout::new::<()>(), 0);
        callback.after_grow(NonNull::dangling(), Layout::new::<()>(), 0, Err(AllocErr));
        callback.before_grow_zeroed(NonNull::dangling(), Layout::new::<()>(), 0);
        callback.after_grow_zeroed(NonNull::dangling(), Layout::new::<()>(), 0, Err(AllocErr));
        callback.before_grow_in_place(NonNull::dangling(), Layout::new::<()>(), 0);
        callback.after_grow_in_place(NonNull::dangling(), Layout::new::<()>(), 0, Err(AllocErr));
        callback.before_grow_in_place_zeroed(NonNull::dangling(), Layout::new::<()>(), 0);
        callback.after_grow_in_place_zeroed(
            NonNull::dangling(),
            Layout::new::<()>(),
            0,
            Err(AllocErr),
        );
        callback.before_shrink(NonNull::dangling(), Layout::new::<()>(), 0);
        callback.after_shrink(NonNull::dangling(), Layout::new::<()>(), 0, Err(AllocErr));
        callback.after_shrink_in_place(NonNull::dangling(), Layout::new::<()>(), 0, Err(AllocErr));
        callback.before_shrink_in_place(NonNull::dangling(), Layout::new::<()>(), 0);
        callback.before_owns();
        callback.after_owns(false);
    }

    fn check_counts(callback: &Callback) {
        assert_eq!(callback.before_alloc.get(), 1);
        assert_eq!(callback.after_alloc.get(), 1);
        assert_eq!(callback.before_alloc_zeroed.get(), 1);
        assert_eq!(callback.after_alloc_zeroed.get(), 1);
        assert_eq!(callback.before_alloc_all.get(), 1);
        assert_eq!(callback.after_alloc_all.get(), 1);
        assert_eq!(callback.before_alloc_all_zeroed.get(), 1);
        assert_eq!(callback.after_alloc_all_zeroed.get(), 1);
        assert_eq!(callback.before_dealloc.get(), 1);
        assert_eq!(callback.after_dealloc.get(), 1);
        assert_eq!(callback.before_dealloc_all.get(), 1);
        assert_eq!(callback.after_dealloc_all.get(), 1);
        assert_eq!(callback.before_grow.get(), 1);
        assert_eq!(callback.after_grow.get(), 1);
        assert_eq!(callback.before_grow_zeroed.get(), 1);
        assert_eq!(callback.after_grow_zeroed.get(), 1);
        assert_eq!(callback.before_grow_in_place.get(), 1);
        assert_eq!(callback.after_grow_in_place.get(), 1);
        assert_eq!(callback.before_grow_in_place_zeroed.get(), 1);
        assert_eq!(callback.after_grow_in_place_zeroed.get(), 1);
        assert_eq!(callback.before_shrink.get(), 1);
        assert_eq!(callback.after_shrink.get(), 1);
        assert_eq!(callback.before_shrink_in_place.get(), 1);
        assert_eq!(callback.after_shrink_in_place.get(), 1);
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
