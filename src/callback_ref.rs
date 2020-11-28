use core::{
    alloc::{AllocError, Layout},
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
    fn before_allocate(&self, layout: Layout) {}

    /// Called after [`alloc`] was invoked.
    ///
    /// [`alloc`]: core::alloc::AllocRef::alloc
    #[inline]
    fn after_allocate(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocError>) {}

    /// Called before [`alloc_zeroed`] was invoked.
    ///
    /// [`alloc_zeroed`]: core::alloc::AllocRef::alloc_zeroed
    #[inline]
    fn before_allocate_zeroed(&self, layout: Layout) {}

    /// Called after [`alloc_zeroed`] was invoked.
    ///
    /// [`alloc_zeroed`]: core::alloc::AllocRef::alloc_zeroed
    #[inline]
    fn after_allocate_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocError>) {}

    /// Called before [`allocate_all`] was invoked.
    ///
    /// [`allocate_all`]: crate::AllocateAll::allocate_all
    #[inline]
    fn before_allocate_all(&self) {}

    /// Called after [`allocate_all`] was invoked.
    ///
    /// [`allocate_all`]: crate::AllocateAll::allocate_all
    #[inline]
    fn after_allocate_all(&self, result: Result<NonNull<[u8]>, AllocError>) {}

    /// Called before [`allocate_all_zeroed`] was invoked.
    ///
    /// [`allocate_all_zeroed`]: crate::AllocateAll::allocate_all_zeroed
    #[inline]
    fn before_allocate_all_zeroed(&self) {}

    /// Called after [`allocate_all_zeroed`] was invoked.
    ///
    /// [`allocate_all_zeroed`]: crate::AllocateAll::allocate_all_zeroed
    #[inline]
    fn after_allocate_all_zeroed(&self, result: Result<NonNull<[u8]>, AllocError>) {}

    /// Called before [`dealloc`] was invoked.
    ///
    /// [`dealloc`]: core::alloc::AllocRef::dealloc
    #[inline]
    fn before_deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}

    /// Called after [`dealloc`] was invoked.
    ///
    /// [`dealloc`]: core::alloc::AllocRef::dealloc
    #[inline]
    fn after_deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}

    /// Called before [`deallocate_all`] was invoked.
    ///
    /// [`deallocate_all`]: crate::AllocateAll::deallocate_all
    #[inline]
    fn before_deallocate_all(&self) {}

    /// Called after [`deallocate_all`] was invoked.
    ///
    /// [`deallocate_all`]: crate::AllocateAll::deallocate_all
    #[inline]
    fn after_deallocate_all(&self) {}

    /// Called before [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    #[inline]
    fn before_grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {}

    /// Called after [`grow`] was invoked.
    ///
    /// [`grow`]: core::alloc::AllocRef::grow
    #[inline]
    fn after_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        result: Result<NonNull<[u8]>, AllocError>,
    ) {
    }

    /// Called before [`grow_zeroed`] was invoked.
    ///
    /// [`grow_zeroed`]: core::alloc::AllocRef::grow_zeroed
    #[inline]
    fn before_grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {}

    /// Called after [`grow_zeroed`] was invoked.
    ///
    /// [`grow_zeroed`]: core::alloc::AllocRef::grow_zeroed
    #[inline]
    fn after_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        result: Result<NonNull<[u8]>, AllocError>,
    ) {
    }

    /// Called before [`grow_in_place`] was invoked.
    ///
    /// [`grow_in_place`]: crate::ReallocateInPlace::grow_in_place
    #[inline]
    fn before_grow_in_place(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {}

    /// Called after [`grow_in_place`] was invoked.
    ///
    /// [`grow_in_place`]: crate::ReallocateInPlace::grow_in_place
    #[inline]
    fn after_grow_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        result: Result<usize, AllocError>,
    ) {
    }

    /// Called before [`grow_in_place_zeroed`] was invoked.
    ///
    /// [`grow_in_place_zeroed`]: crate::ReallocateInPlace::grow_in_place_zeroed
    #[inline]
    fn before_grow_in_place_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) {
    }

    /// Called after [`grow_in_place_zeroed`] was invoked.
    ///
    /// [`grow_in_place_zeroed`]: crate::ReallocateInPlace::grow_in_place_zeroed
    #[inline]
    fn after_grow_in_place_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        result: Result<usize, AllocError>,
    ) {
    }

    /// Called before [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    #[inline]
    fn before_shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {}

    /// Called after [`shrink`] was invoked.
    ///
    /// [`shrink`]: core::alloc::AllocRef::shrink
    #[inline]
    fn after_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        result: Result<NonNull<[u8]>, AllocError>,
    ) {
    }

    /// Called before [`shrink_in_place`] was invoked.
    ///
    /// [`shrink_in_place`]: crate::ReallocateInPlace::shrink_in_place
    #[inline]
    fn before_shrink_in_place(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {}

    /// Called after [`shrink_in_place`] was invoked.
    ///
    /// [`shrink_in_place`]: crate::ReallocateInPlace::shrink_in_place
    #[inline]
    fn after_shrink_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        result: Result<usize, AllocError>,
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
    ($(#[$meta:meta])* $ty:ty) => {
        $(#[$meta])*
        unsafe impl<C> CallbackRef for $ty where C: CallbackRef + ?Sized {
            #[inline]
            fn before_allocate(&self, layout: Layout) {
                (**self).before_allocate(layout)
            }

            #[inline]
            fn after_allocate(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocError>) {
                (**self).after_allocate(layout, result)
            }

            #[inline]
            fn before_allocate_zeroed(&self, layout: Layout) {
                (**self).before_allocate_zeroed(layout)
            }

            #[inline]
            fn after_allocate_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocError>) {
                (**self).after_allocate_zeroed(layout, result)
            }

            #[inline]
            fn before_allocate_all(&self) {
                (**self).before_allocate_all()
            }

            #[inline]
            fn after_allocate_all(&self, result: Result<NonNull<[u8]>, AllocError>) {
                (**self).after_allocate_all(result)
            }

            #[inline]
            fn before_allocate_all_zeroed(&self) {
                (**self).before_allocate_all_zeroed()
            }

            #[inline]
            fn after_allocate_all_zeroed(
                &self,
                result: Result<NonNull<[u8]>, AllocError>,
            ) {
                (**self).after_allocate_all_zeroed(result)
            }

            #[inline]
            fn before_deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
                (**self).before_deallocate(ptr, layout)
            }

            #[inline]
            fn after_deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
                (**self).after_deallocate(ptr, layout)
            }

            #[inline]
            fn before_deallocate_all(&self) {
                (**self).before_deallocate_all()
            }

            #[inline]
            fn after_deallocate_all(&self) {
                (**self).after_deallocate_all()
            }

            #[inline]
            fn before_grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
                (**self).before_grow(ptr, old_layout, new_layout)
            }

            #[inline]
            fn after_grow(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
                result: Result<NonNull<[u8]>, AllocError>,
            ) {
                (**self).after_grow(ptr, old_layout, new_layout, result)
            }

            #[inline]
            fn before_grow_zeroed(&self, ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,) {
                (**self).before_grow_zeroed(ptr, old_layout, new_layout)
            }

            #[inline]
            fn after_grow_zeroed(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
                result: Result<NonNull<[u8]>, AllocError>,
            ) {
                (**self).after_grow_zeroed(ptr, old_layout, new_layout, result)
            }

            #[inline]
            fn before_grow_in_place(&self, ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,) {
                (**self).before_grow_in_place(ptr, old_layout, new_layout)
            }

            #[inline]
            fn after_grow_in_place(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
                result: Result<usize, AllocError>,
            ) {
                (**self).after_grow_in_place(ptr, old_layout, new_layout, result)
            }

            #[inline]
            fn before_grow_in_place_zeroed(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
            ) {
                (**self).before_grow_in_place_zeroed(ptr, old_layout, new_layout)
            }

            #[inline]
            fn after_grow_in_place_zeroed(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
                result: Result<usize, AllocError>,
            ) {
                (**self).after_grow_in_place_zeroed(ptr, old_layout, new_layout, result)
            }

            #[inline]
            fn before_shrink(&self, ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,) {
                (**self).before_shrink(ptr, old_layout, new_layout)
            }

            #[inline]
            fn after_shrink(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
                result: Result<NonNull<[u8]>, AllocError>,
            ) {
                (**self).after_shrink(ptr, old_layout, new_layout, result)
            }

            #[inline]
            fn before_shrink_in_place(&self, ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,) {
                (**self).before_shrink_in_place(ptr, old_layout, new_layout)
            }

            #[inline]
            fn after_shrink_in_place(
                &self,
                ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
                result: Result<usize, AllocError>,
            ) {
                (**self).after_shrink_in_place(ptr, old_layout, new_layout, result)
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
impl_alloc_stats!(#[cfg_attr(doc, doc(cfg(feature = "alloc")))] alloc::boxed::Box<C>);
#[cfg(any(doc, feature = "alloc"))]
impl_alloc_stats!(#[cfg_attr(doc, doc(cfg(feature = "alloc")))] alloc::rc::Rc<C>);
#[cfg(any(doc, feature = "alloc"))]
impl_alloc_stats!(#[cfg_attr(doc, doc(cfg(feature = "alloc")))] alloc::sync::Arc<C>);

#[cfg(test)]
mod tests {
    use crate::CallbackRef;
    use alloc::{boxed::Box, rc::Rc, sync::Arc};
    use core::{
        alloc::{AllocError, Layout},
        cell::Cell,
        ptr::NonNull,
    };

    #[derive(Default)]
    struct Callback {
        before_allocate: Cell<u32>,
        after_allocate: Cell<u32>,
        before_allocate_zeroed: Cell<u32>,
        after_allocate_zeroed: Cell<u32>,
        before_allocate_all: Cell<u32>,
        after_allocate_all: Cell<u32>,
        before_allocate_all_zeroed: Cell<u32>,
        after_allocate_all_zeroed: Cell<u32>,
        before_deallocate: Cell<u32>,
        after_deallocate: Cell<u32>,
        before_deallocate_all: Cell<u32>,
        after_deallocate_all: Cell<u32>,
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
        fn before_allocate(&self, _layout: Layout) {
            self.before_allocate.set(self.before_allocate.get() + 1)
        }
        fn after_allocate(&self, _layout: Layout, _result: Result<NonNull<[u8]>, AllocError>) {
            self.after_allocate.set(self.after_allocate.get() + 1)
        }
        fn before_allocate_zeroed(&self, _layout: Layout) {
            self.before_allocate_zeroed
                .set(self.before_allocate_zeroed.get() + 1)
        }
        fn after_allocate_zeroed(
            &self,
            _layout: Layout,
            _result: Result<NonNull<[u8]>, AllocError>,
        ) {
            self.after_allocate_zeroed
                .set(self.after_allocate_zeroed.get() + 1)
        }
        fn before_allocate_all(&self) {
            self.before_allocate_all
                .set(self.before_allocate_all.get() + 1)
        }
        fn after_allocate_all(&self, _result: Result<NonNull<[u8]>, AllocError>) {
            self.after_allocate_all
                .set(self.after_allocate_all.get() + 1)
        }
        fn before_allocate_all_zeroed(&self) {
            self.before_allocate_all_zeroed
                .set(self.before_allocate_all_zeroed.get() + 1)
        }
        fn after_allocate_all_zeroed(&self, _result: Result<NonNull<[u8]>, AllocError>) {
            self.after_allocate_all_zeroed
                .set(self.after_allocate_all_zeroed.get() + 1)
        }
        fn before_deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
            self.before_deallocate.set(self.before_deallocate.get() + 1)
        }
        fn after_deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
            self.after_deallocate.set(self.after_deallocate.get() + 1)
        }
        fn before_deallocate_all(&self) {
            self.before_deallocate_all
                .set(self.before_deallocate_all.get() + 1)
        }
        fn after_deallocate_all(&self) {
            self.after_deallocate_all
                .set(self.after_deallocate_all.get() + 1)
        }
        fn before_grow(&self, _ptr: NonNull<u8>, _old_layout: Layout, _new_layout: Layout) {
            self.before_grow.set(self.before_grow.get() + 1)
        }
        fn after_grow(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
            _result: Result<NonNull<[u8]>, AllocError>,
        ) {
            self.after_grow.set(self.after_grow.get() + 1)
        }
        fn before_grow_zeroed(&self, _ptr: NonNull<u8>, _old_layout: Layout, _new_layout: Layout) {
            self.before_grow_zeroed
                .set(self.before_grow_zeroed.get() + 1)
        }
        fn after_grow_zeroed(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
            _result: Result<NonNull<[u8]>, AllocError>,
        ) {
            self.after_grow_zeroed.set(self.after_grow_zeroed.get() + 1)
        }
        fn before_grow_in_place(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
        ) {
            self.before_grow_in_place
                .set(self.before_grow_in_place.get() + 1)
        }
        fn after_grow_in_place(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
            _result: Result<usize, AllocError>,
        ) {
            self.after_grow_in_place
                .set(self.after_grow_in_place.get() + 1)
        }
        fn before_grow_in_place_zeroed(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
        ) {
            self.before_grow_in_place_zeroed
                .set(self.before_grow_in_place_zeroed.get() + 1)
        }
        fn after_grow_in_place_zeroed(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
            _result: Result<usize, AllocError>,
        ) {
            self.after_grow_in_place_zeroed
                .set(self.after_grow_in_place_zeroed.get() + 1)
        }
        fn before_shrink(&self, _ptr: NonNull<u8>, _old_layout: Layout, _new_layout: Layout) {
            self.before_shrink.set(self.before_shrink.get() + 1)
        }
        fn after_shrink(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
            _result: Result<NonNull<[u8]>, AllocError>,
        ) {
            self.after_shrink.set(self.after_shrink.get() + 1)
        }
        fn before_shrink_in_place(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
        ) {
            self.before_shrink_in_place
                .set(self.before_shrink_in_place.get() + 1)
        }
        fn after_shrink_in_place(
            &self,
            _ptr: NonNull<u8>,
            _old_layout: Layout,
            _new_layout: Layout,
            _result: Result<usize, AllocError>,
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
        callback.before_allocate(Layout::new::<()>());
        callback.after_allocate(Layout::new::<()>(), Err(AllocError));
        callback.before_allocate_zeroed(Layout::new::<()>());
        callback.after_allocate_zeroed(Layout::new::<()>(), Err(AllocError));
        callback.before_allocate_all();
        callback.after_allocate_all(Err(AllocError));
        callback.before_allocate_all_zeroed();
        callback.after_allocate_all_zeroed(Err(AllocError));
        callback.before_deallocate(NonNull::dangling(), Layout::new::<()>());
        callback.after_deallocate(NonNull::dangling(), Layout::new::<()>());
        callback.before_deallocate_all();
        callback.after_deallocate_all();
        callback.before_grow(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
        );
        callback.after_grow(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
            Err(AllocError),
        );
        callback.before_grow_zeroed(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
        );
        callback.after_grow_zeroed(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
            Err(AllocError),
        );
        callback.before_grow_in_place(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
        );
        callback.after_grow_in_place(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
            Err(AllocError),
        );
        callback.before_grow_in_place_zeroed(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
        );
        callback.after_grow_in_place_zeroed(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
            Err(AllocError),
        );
        callback.before_shrink(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
        );
        callback.after_shrink(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
            Err(AllocError),
        );
        callback.after_shrink_in_place(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
            Err(AllocError),
        );
        callback.before_shrink_in_place(
            NonNull::dangling(),
            Layout::new::<()>(),
            Layout::new::<()>(),
        );
        callback.before_owns();
        callback.after_owns(false);
    }

    fn check_counts(callback: &Callback) {
        assert_eq!(callback.before_allocate.get(), 1);
        assert_eq!(callback.after_allocate.get(), 1);
        assert_eq!(callback.before_allocate_zeroed.get(), 1);
        assert_eq!(callback.after_allocate_zeroed.get(), 1);
        assert_eq!(callback.before_allocate_all.get(), 1);
        assert_eq!(callback.after_allocate_all.get(), 1);
        assert_eq!(callback.before_allocate_all_zeroed.get(), 1);
        assert_eq!(callback.after_allocate_all_zeroed.get(), 1);
        assert_eq!(callback.before_deallocate.get(), 1);
        assert_eq!(callback.after_deallocate.get(), 1);
        assert_eq!(callback.before_deallocate_all.get(), 1);
        assert_eq!(callback.after_deallocate_all.get(), 1);
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
