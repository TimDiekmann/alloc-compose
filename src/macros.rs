macro_rules! impl_alloc_ref {
    ($parent:tt) => {
        fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
            Self::alloc_impl(layout, |l| self.$parent.alloc(l))
        }

        fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
            Self::alloc_impl(layout, |l| self.$parent.alloc_zeroed(l))
        }

        unsafe fn grow(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<NonNull<[u8]>, AllocErr> {
            crate::check_grow_precondition(ptr, layout, new_size);
            Self::grow_impl(
                ptr,
                layout,
                new_size,
                AllocInit::Uninitialized,
                |ptr, layout, new_size| self.$parent.grow(ptr, layout, new_size),
            )
        }

        unsafe fn grow_zeroed(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<NonNull<[u8]>, AllocErr> {
            crate::check_grow_precondition(ptr, layout, new_size);
            Self::grow_impl(
                ptr,
                layout,
                new_size,
                AllocInit::Zeroed,
                |ptr, layout, new_size| self.$parent.grow_zeroed(ptr, layout, new_size),
            )
        }

        unsafe fn shrink(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<NonNull<[u8]>, AllocErr> {
            crate::check_shrink_precondition(ptr, layout, new_size);
            Self::shrink_impl(ptr, layout, new_size, |ptr, layout, new_size| {
                self.$parent.shrink(ptr, layout, new_size)
            })
        }
    };
}
macro_rules! impl_alloc_all {
    ($parent:tt) => {
        fn alloc_all(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
            Self::alloc_impl(layout, |layout| self.$parent.alloc_all(layout))
        }

        fn alloc_all_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
            Self::alloc_impl(layout, |layout| self.$parent.alloc_all_zeroed(layout))
        }

        fn dealloc_all(&mut self) {
            self.$parent.dealloc_all()
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
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<usize, AllocErr> {
            crate::check_grow_precondition(ptr, layout, new_size);
            Self::grow_impl(
                ptr,
                layout,
                new_size,
                AllocInit::Uninitialized,
                |ptr, layout, new_size| {
                    crate::check_grow_precondition(ptr, layout, new_size);
                    self.$parent
                        .grow_in_place(ptr, layout, new_size)
                        .map(|len| NonNull::slice_from_raw_parts(ptr, len))
                },
            )
            .map(NonNull::len)
        }

        unsafe fn grow_in_place_zeroed(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<usize, AllocErr> {
            crate::check_grow_precondition(ptr, layout, new_size);
            Self::grow_impl(
                ptr,
                layout,
                new_size,
                AllocInit::Zeroed,
                |ptr, layout, new_size| {
                    crate::check_grow_precondition(ptr, layout, new_size);
                    self.$parent
                        .grow_in_place_zeroed(ptr, layout, new_size)
                        .map(|len| NonNull::slice_from_raw_parts(ptr, len))
                },
            )
            .map(NonNull::len)
        }

        unsafe fn shrink_in_place(
            &mut self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
        ) -> Result<usize, AllocErr> {
            crate::check_shrink_precondition(ptr, layout, new_size);
            Self::shrink_impl(ptr, layout, new_size, |ptr, layout, new_size| {
                crate::check_shrink_precondition(ptr, layout, new_size);
                self.$parent
                    .shrink_in_place(ptr, layout, new_size)
                    .map(|len| NonNull::slice_from_raw_parts(ptr, len))
            })
            .map(NonNull::len)
        }
    };
}
