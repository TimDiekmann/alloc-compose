macro_rules! impl_global_alloc {
    ($ty:path) => {
        unsafe impl core::alloc::GlobalAlloc for $ty {
            unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
                core::alloc::AllocRef::alloc(&self, layout)
                    .map(core::ptr::NonNull::as_mut_ptr)
                    .unwrap_or(core::ptr::null_mut())
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
                core::alloc::AllocRef::dealloc(
                    &self,
                    core::ptr::NonNull::new_unchecked(ptr),
                    layout,
                )
            }

            unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
                core::alloc::AllocRef::alloc_zeroed(&self, layout)
                    .map(core::ptr::NonNull::as_mut_ptr)
                    .unwrap_or(core::ptr::null_mut())
            }

            unsafe fn realloc(
                &self,
                ptr: *mut u8,
                layout: core::alloc::Layout,
                new_size: usize,
            ) -> *mut u8 {
                if new_size > layout.size() {
                    core::alloc::AllocRef::grow(
                        &self,
                        core::ptr::NonNull::new_unchecked(ptr),
                        layout,
                        core::alloc::Layout::from_size_align_unchecked(new_size, layout.align()),
                    )
                    .map(core::ptr::NonNull::as_mut_ptr)
                    .unwrap_or(core::ptr::null_mut())
                } else {
                    core::alloc::AllocRef::shrink(
                        &self,
                        core::ptr::NonNull::new_unchecked(ptr),
                        layout,
                        core::alloc::Layout::from_size_align_unchecked(new_size, layout.align()),
                    )
                    .map(core::ptr::NonNull::as_mut_ptr)
                    .unwrap_or(core::ptr::null_mut())
                }
            }
        }
    };
}

macro_rules! impl_alloc_ref {
    ($parent:tt) => {
        fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            Self::alloc_impl(layout, |l| self.$parent.alloc(l))
        }

        fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            Self::alloc_impl(layout, |l| self.$parent.alloc_zeroed(l))
        }

        unsafe fn grow(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            crate::check_grow_precondition(ptr, old_layout, new_layout);
            Self::grow_impl(
                ptr,
                old_layout,
                new_layout,
                AllocInit::Uninitialized,
                |ptr, old_layout, new_layout| self.$parent.grow(ptr, old_layout, new_layout),
            )
        }

        unsafe fn grow_zeroed(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            crate::check_grow_precondition(ptr, old_layout, new_layout);
            Self::grow_impl(
                ptr,
                old_layout,
                new_layout,
                AllocInit::Zeroed,
                |ptr, old_layout, new_layout| self.$parent.grow_zeroed(ptr, old_layout, new_layout),
            )
        }

        unsafe fn shrink(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            crate::check_shrink_precondition(ptr, old_layout, new_layout);
            Self::shrink_impl(
                ptr,
                old_layout,
                new_layout,
                |ptr, old_layout, new_layout| self.$parent.shrink(ptr, old_layout, new_layout),
            )
        }
    };
}
macro_rules! impl_alloc_all {
    ($parent:tt) => {
        fn allocate_all(&self) -> Result<NonNull<[u8]>, AllocError> {
            Self::allocate_all_impl(|| self.$parent.allocate_all())
        }

        fn allocate_all_zeroed(&self) -> Result<NonNull<[u8]>, AllocError> {
            Self::allocate_all_impl(|| self.$parent.allocate_all_zeroed())
        }

        fn deallocate_all(&self) {
            self.$parent.deallocate_all()
        }

        fn capacity(&self) -> usize {
            self.$parent.capacity()
        }

        fn capacity_left(&self) -> usize {
            self.$parent.capacity_left()
        }
    };
}

macro_rules! impl_realloc_in_place {
    ($parent:tt) => {
        unsafe fn grow_in_place(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<usize, AllocError> {
            crate::check_grow_precondition(ptr, old_layout, new_layout);
            Self::grow_impl(
                ptr,
                old_layout,
                new_layout,
                AllocInit::Uninitialized,
                |ptr, old_layout, new_layout| {
                    crate::check_grow_precondition(ptr, old_layout, new_layout);
                    self.$parent
                        .grow_in_place(ptr, old_layout, new_layout)
                        .map(|len| NonNull::slice_from_raw_parts(ptr, len))
                },
            )
            .map(NonNull::len)
        }

        unsafe fn grow_in_place_zeroed(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<usize, AllocError> {
            crate::check_grow_precondition(ptr, old_layout, new_layout);
            Self::grow_impl(
                ptr,
                old_layout,
                new_layout,
                AllocInit::Zeroed,
                |ptr, old_layout, new_layout| {
                    crate::check_grow_precondition(ptr, old_layout, new_layout);
                    self.$parent
                        .grow_in_place_zeroed(ptr, old_layout, new_layout)
                        .map(|len| NonNull::slice_from_raw_parts(ptr, len))
                },
            )
            .map(NonNull::len)
        }

        unsafe fn shrink_in_place(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<usize, AllocError> {
            crate::check_shrink_precondition(ptr, old_layout, new_layout);
            Self::shrink_impl(
                ptr,
                old_layout,
                new_layout,
                |ptr, old_layout, new_layout| {
                    crate::check_shrink_precondition(ptr, old_layout, new_layout);
                    self.$parent
                        .shrink_in_place(ptr, old_layout, new_layout)
                        .map(|len| NonNull::slice_from_raw_parts(ptr, len))
                },
            )
            .map(NonNull::len)
        }
    };
}

macro_rules! impl_realloc_in_place_spec {
    ($parent:tt) => {
        default unsafe fn grow_in_place(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<usize, AllocError> {
            crate::check_grow_precondition(ptr, old_layout, new_layout);
            Self::grow_impl(
                ptr,
                old_layout,
                new_layout,
                AllocInit::Uninitialized,
                |ptr, old_layout, new_layout| {
                    crate::check_grow_precondition(ptr, old_layout, new_layout);
                    Err(AllocError)
                },
            )
            .map(NonNull::len)
        }

        default unsafe fn grow_in_place_zeroed(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<usize, AllocError> {
            crate::check_grow_precondition(ptr, old_layout, new_layout);
            Self::grow_impl(
                ptr,
                old_layout,
                new_layout,
                AllocInit::Zeroed,
                |ptr, old_layout, new_layout| {
                    crate::check_grow_precondition(ptr, old_layout, new_layout);
                    Err(AllocError)
                },
            )
            .map(NonNull::len)
        }

        default unsafe fn shrink_in_place(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<usize, AllocError> {
            crate::check_shrink_precondition(ptr, old_layout, new_layout);
            Self::shrink_impl(
                ptr,
                old_layout,
                new_layout,
                |ptr, old_layout, new_layout| {
                    crate::check_shrink_precondition(ptr, old_layout, new_layout);
                    Err(AllocError)
                },
            )
            .map(NonNull::len)
        }
    };
}
