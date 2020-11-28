use core::{
    alloc::{AllocError, AllocRef, Layout},
    ptr::{self, NonNull},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AllocInit {
    Uninitialized,
    Zeroed,
}

impl AllocInit {
    #[inline]
    pub unsafe fn init_offset(self, ptr: NonNull<[u8]>, offset: usize) {
        debug_assert!(
            offset <= ptr.len(),
            "`offset` must be smaller than or equal to `ptr.len()`"
        );
        match self {
            Self::Uninitialized => (),
            Self::Zeroed => ptr
                .as_non_null_ptr()
                .as_ptr()
                .add(offset)
                .write_bytes(0, ptr.len() - offset),
        }
    }
}

// #[derive(Copy, Clone, PartialEq, Eq)]
// pub enum ReallocPlacement {
//     MayMove,
//     InPlace,
// }

pub(in crate) unsafe fn grow_fallback<A1: AllocRef, A2: AllocRef>(
    a1: &mut A1,
    a2: &mut A2,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
    init: AllocInit,
) -> Result<NonNull<[u8]>, AllocError> {
    let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
    let new_ptr = match init {
        AllocInit::Uninitialized => a2.alloc(new_layout)?,
        AllocInit::Zeroed => a2.alloc_zeroed(new_layout)?,
    };
    ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), layout.size());
    a1.dealloc(ptr, layout);
    Ok(new_ptr)
}

pub(in crate) unsafe fn shrink_fallback<A1: AllocRef, A2: AllocRef>(
    a1: &mut A1,
    a2: &mut A2,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
) -> Result<NonNull<[u8]>, AllocError> {
    let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
    let new_ptr = a2.alloc(new_layout)?;
    ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_size);
    a1.dealloc(ptr, layout);
    Ok(new_ptr)
}

#[cfg(test)]
pub fn tracker<A: AllocRef>(alloc: A) -> crate::Proxy<A, impl crate::CallbackRef> {
    crate::Proxy {
        alloc,
        callbacks: self::tests::Tracker::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::tracker;
    use crate::{CallbackRef, Chunk};
    use alloc::{alloc::Global, collections::BTreeMap};
    use core::{
        alloc::{AllocError, AllocRef, Layout},
        cell::RefCell,
        ptr::NonNull,
    };

    #[cfg(test)]
    #[derive(Default)]
    pub struct Tracker {
        map: RefCell<BTreeMap<NonNull<u8>, (usize, Layout)>>,
    }

    impl Tracker {
        fn assert_fit_memory(&self, ptr: NonNull<u8>, layout: Layout, name: &str) {
            let map = self.map.borrow();
            let (size, old_layout) = map.get(&ptr).expect(
                "`ptr` must denote a block of memory currently allocated via this allocator",
            );
            assert_eq!(
                layout.align(),
                old_layout.align(),
                "`{0}` must fit that block of memory. The block must be allocated with the same \
                 alignment as `{1}.align()`. Expected alignment of {1}, got {2}",
                name,
                old_layout.align(),
                layout.align()
            );
            if *size == old_layout.size() {
                assert_eq!(
                    layout.size(),
                    old_layout.size(),
                    "`{0}` must fit that block of memory. The provided `{0}.size()` must fall in \
                     the range `min ..= max`. Expected size of {1}, got {2}",
                    name,
                    old_layout.size(),
                    layout.size()
                )
            } else {
                assert!(
                    layout.size() >= *size && layout.size() <= old_layout.size(),
                    "`{0}` must fit that block of memory. The provided `{0}.size()` must fall in \
                     the range `min ..= max`. Expected size between `{1} ..= {2}`, got {3}",
                    name,
                    size,
                    old_layout.size(),
                    layout.size()
                )
            }
        }
    }

    #[cfg(test)]
    unsafe impl CallbackRef for Tracker {
        fn after_allocate(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocError>) {
            if let Ok(ptr) = result {
                self.map
                    .borrow_mut()
                    .insert(ptr.as_non_null_ptr(), (ptr.len(), layout));
            }
        }

        fn after_allocate_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocError>) {
            self.after_allocate(layout, result)
        }

        fn after_allocate_all(&self, result: Result<NonNull<[u8]>, AllocError>) {
            if let Ok(ptr) = result {
                let layout =
                    Layout::from_size_align(ptr.len(), 1).expect("Invalid layout for allocate_all");
                self.after_allocate(layout, result);
            }
        }

        fn after_allocate_all_zeroed(&self, result: Result<NonNull<[u8]>, AllocError>) {
            self.after_allocate_all(result)
        }

        #[track_caller]
        fn before_deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            self.assert_fit_memory(ptr, layout, "layout");
        }

        fn after_deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
            let mut map = self.map.borrow_mut();
            map.remove(&ptr);
        }

        fn after_deallocate_all(&self) {
            let mut map = self.map.borrow_mut();
            map.clear()
        }

        #[track_caller]
        fn before_grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
            self.assert_fit_memory(ptr, old_layout, "old_layout");
            assert!(
                new_layout.size() >= old_layout.size(),
                "`new_layout.size()` must be greater than or equal to `old_layout.size()`, \
                 expected {} >= {}",
                new_layout.size(),
                old_layout.size()
            );
        }

        fn after_grow(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
            result: Result<NonNull<[u8]>, AllocError>,
        ) {
            if result.is_ok() {
                self.after_deallocate(ptr, old_layout);
                self.after_allocate(new_layout, result);
            }
        }

        #[track_caller]
        fn before_grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
            self.before_grow(ptr, old_layout, new_layout)
        }

        fn after_grow_zeroed(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
            result: Result<NonNull<[u8]>, AllocError>,
        ) {
            self.after_grow(ptr, old_layout, new_layout, result)
        }

        #[track_caller]
        fn before_grow_in_place(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
            self.before_grow(ptr, old_layout, new_layout)
        }

        fn after_grow_in_place(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
            result: Result<usize, AllocError>,
        ) {
            self.after_grow(
                ptr,
                old_layout,
                new_layout,
                result.map(|len| NonNull::slice_from_raw_parts(ptr, len)),
            )
        }

        #[track_caller]
        fn before_grow_in_place_zeroed(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) {
            self.before_grow_in_place(ptr, old_layout, new_layout)
        }

        fn after_grow_in_place_zeroed(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
            result: Result<usize, AllocError>,
        ) {
            self.after_grow_in_place(ptr, old_layout, new_layout, result)
        }

        #[track_caller]
        fn before_shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
            self.assert_fit_memory(ptr, old_layout, "old_layout");
            assert!(
                new_layout.size() <= old_layout.size(),
                "`new_layout.size()` must be smaller than or equal to `old_layout.size()`, \
                 expected {} <= {}",
                new_layout.size(),
                old_layout.size()
            );
        }

        fn after_shrink(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
            result: Result<NonNull<[u8]>, AllocError>,
        ) {
            if result.is_ok() {
                self.after_deallocate(ptr, old_layout);
                self.after_allocate(new_layout, result);
            }
        }

        #[track_caller]
        fn before_shrink_in_place(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) {
            self.before_shrink(ptr, old_layout, new_layout)
        }

        fn after_shrink_in_place(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
            result: Result<usize, AllocError>,
        ) {
            self.after_shrink(
                ptr,
                old_layout,
                new_layout,
                result.map(|len| NonNull::slice_from_raw_parts(ptr, len)),
            )
        }
    }

    struct DeallocGuard<A: AllocRef> {
        allocator: A,
        ptr: NonNull<u8>,
        layout: Layout,
    }

    impl<A: AllocRef> DeallocGuard<A> {
        fn new(allocator: A, ptr: NonNull<[u8]>, layout: Layout) -> Self {
            Self {
                allocator,
                ptr: ptr.as_non_null_ptr(),
                layout,
            }
        }
    }

    impl<A: AllocRef> Drop for DeallocGuard<A> {
        fn drop(&mut self) {
            unsafe { self.allocator.dealloc(self.ptr, self.layout) }
        }
    }

    #[test]
    #[should_panic = "`new_layout.size()` must be greater than or equal to `old_layout.size()`"]
    fn tracker_grow_size_greater_layout() {
        let alloc = tracker(Global);
        let layout = Layout::new::<[u8; 4]>();
        let memory = alloc.alloc(layout).expect("Could not allocate 4 bytes");
        let _guard = DeallocGuard::new(Global, memory, layout);
        let _ = unsafe { alloc.grow(memory.as_non_null_ptr(), layout, Layout::new::<[u8; 2]>()) };
    }

    #[test]
    #[should_panic = "`old_layout` must fit that block of memory"]
    fn tracker_grow_layout_size_exact() {
        let alloc = tracker(Global);
        let layout = Layout::new::<[u8; 4]>();
        let memory = alloc.alloc(layout).expect("Could not allocate 4 bytes");
        let _guard = DeallocGuard::new(Global, memory, layout);
        let _ = unsafe {
            alloc.grow(
                memory.as_non_null_ptr(),
                Layout::new::<[u8; 2]>(),
                Layout::new::<[u8; 10]>(),
            )
        };
    }

    #[test]
    #[should_panic = "`old_layout` must fit that block of memory"]
    fn tracker_grow_layout_size_range() {
        let alloc = tracker(Chunk::<Global, 32>::default());
        let layout = Layout::new::<[u8; 4]>();
        let memory = alloc.alloc(layout).expect("Could not allocate 4 bytes");
        let _guard = DeallocGuard::new(Chunk::<Global, 32>::default(), memory, layout);
        let _ = unsafe {
            alloc.grow(
                memory.as_non_null_ptr(),
                Layout::new::<[u8; 2]>(),
                Layout::new::<[u8; 10]>(),
            )
        };
    }

    #[test]
    #[should_panic = "`old_layout` must fit that block of memory"]
    fn tracker_grow_layout_align() {
        let alloc = tracker(Global);
        let layout = Layout::new::<[u8; 4]>();
        let memory = alloc.alloc(layout).expect("Could not allocate 4 bytes");
        let _guard = DeallocGuard::new(Global, memory, layout);
        let _ = unsafe {
            alloc.grow(
                memory.as_non_null_ptr(),
                Layout::new::<[u16; 2]>(),
                Layout::new::<[u8; 4]>(),
            )
        };
    }

    #[test]
    #[should_panic = "`ptr` must denote a block of memory currently allocated via this allocator"]
    fn tracker_grow_ptr() {
        let alloc = tracker(Global);
        let layout = Layout::new::<[u8; 4]>();
        let memory = alloc.alloc(layout).expect("Could not allocate 4 bytes");
        let _guard = DeallocGuard::new(Global, memory, layout);
        let _ = unsafe {
            alloc.grow(
                NonNull::dangling(),
                Layout::new::<[u8; 4]>(),
                Layout::new::<[u8; 10]>(),
            )
        };
    }
}
