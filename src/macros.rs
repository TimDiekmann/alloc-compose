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

// macro_rules! impl_alloc_ref {
//     ($parent:tt) => {
//         fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
//             Self::alloc_impl(layout, |l| self.$parent.alloc(l))
//         }

//         fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
//             Self::alloc_impl(layout, |l| self.$parent.alloc_zeroed(l))
//         }

//         unsafe fn grow(
//             &mut self,
//             ptr: NonNull<u8>,
//             layout: Layout,
//             new_size: usize,
//         ) -> Result<NonNull<[u8]>, AllocErr> {
//             crate::check_grow_precondition(ptr, layout, new_size);
//             Self::grow_impl(
//                 ptr,
//                 layout,
//                 new_size,
//                 AllocInit::Uninitialized,
//                 |ptr, layout, new_size| self.$parent.grow(ptr, layout, new_size),
//             )
//         }

//         unsafe fn grow_zeroed(
//             &mut self,
//             ptr: NonNull<u8>,
//             layout: Layout,
//             new_size: usize,
//         ) -> Result<NonNull<[u8]>, AllocErr> {
//             crate::check_grow_precondition(ptr, layout, new_size);
//             Self::grow_impl(
//                 ptr,
//                 layout,
//                 new_size,
//                 AllocInit::Zeroed,
//                 |ptr, layout, new_size| self.$parent.grow_zeroed(ptr, layout, new_size),
//             )
//         }

//         unsafe fn shrink(
//             &mut self,
//             ptr: NonNull<u8>,
//             layout: Layout,
//             new_size: usize,
//         ) -> Result<NonNull<[u8]>, AllocErr> {
//             crate::check_shrink_precondition(ptr, layout, new_size);
//             Self::shrink_impl(ptr, layout, new_size, |ptr, layout, new_size| {
//                 self.$parent.shrink(ptr, layout, new_size)
//             })
//         }
//     };
// }
// macro_rules! impl_alloc_all {
//     ($parent:tt) => {
//         fn alloc_all(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
//             Self::alloc_impl(layout, |layout| self.$parent.alloc_all(layout))
//         }

//         fn alloc_all_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
//             Self::alloc_impl(layout, |layout| self.$parent.alloc_all_zeroed(layout))
//         }

//         fn dealloc_all(&mut self) {
//             self.$parent.dealloc_all()
//         }

//         fn capacity(&self) -> usize {
//             self.$parent.capacity()
//         }

//         fn capacity_left(&self) -> usize {
//             self.$parent.capacity_left()
//         }
//     };
// }

// macro_rules! impl_realloc_in_place {
//     ($parent:tt) => {
//         unsafe fn grow_in_place(
//             &mut self,
//             ptr: NonNull<u8>,
//             layout: Layout,
//             new_size: usize,
//         ) -> Result<usize, AllocErr> {
//             crate::check_grow_precondition(ptr, layout, new_size);
//             Self::grow_impl(
//                 ptr,
//                 layout,
//                 new_size,
//                 AllocInit::Uninitialized,
//                 |ptr, layout, new_size| {
//                     crate::check_grow_precondition(ptr, layout, new_size);
//                     self.$parent
//                         .grow_in_place(ptr, layout, new_size)
//                         .map(|len| NonNull::slice_from_raw_parts(ptr, len))
//                 },
//             )
//             .map(NonNull::len)
//         }

//         unsafe fn grow_in_place_zeroed(
//             &mut self,
//             ptr: NonNull<u8>,
//             layout: Layout,
//             new_size: usize,
//         ) -> Result<usize, AllocErr> {
//             crate::check_grow_precondition(ptr, layout, new_size);
//             Self::grow_impl(
//                 ptr,
//                 layout,
//                 new_size,
//                 AllocInit::Zeroed,
//                 |ptr, layout, new_size| {
//                     crate::check_grow_precondition(ptr, layout, new_size);
//                     self.$parent
//                         .grow_in_place_zeroed(ptr, layout, new_size)
//                         .map(|len| NonNull::slice_from_raw_parts(ptr, len))
//                 },
//             )
//             .map(NonNull::len)
//         }

//         unsafe fn shrink_in_place(
//             &mut self,
//             ptr: NonNull<u8>,
//             layout: Layout,
//             new_size: usize,
//         ) -> Result<usize, AllocErr> {
//             crate::check_shrink_precondition(ptr, layout, new_size);
//             Self::shrink_impl(ptr, layout, new_size, |ptr, layout, new_size| {
//                 crate::check_shrink_precondition(ptr, layout, new_size);
//                 self.$parent
//                     .shrink_in_place(ptr, layout, new_size)
//                     .map(|len| NonNull::slice_from_raw_parts(ptr, len))
//             })
//             .map(NonNull::len)
//         }
//     };
// }
