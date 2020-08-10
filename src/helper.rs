use core::{
    alloc::{AllocErr, AllocRef, Layout},
    ptr::{self, NonNull},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AllocInit {
    Uninitialized,
    Zeroed,
}

impl AllocInit {
    #[inline]
    pub unsafe fn init(self, ptr: NonNull<[u8]>) {
        self.init_offset(ptr, 0)
    }

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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReallocPlacement {
    MayMove,
    InPlace,
}

pub(in crate) unsafe fn grow_fallback<A1: AllocRef, A2: AllocRef>(
    a1: &mut A1,
    a2: &mut A2,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
    init: AllocInit,
) -> Result<NonNull<[u8]>, AllocErr> {
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
) -> Result<NonNull<[u8]>, AllocErr> {
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
    use std::{
        alloc::{AllocErr, AllocRef, Layout, System},
        collections::HashMap,
        ptr::NonNull,
        sync::{Mutex, PoisonError},
    };

    #[cfg(test)]
    #[derive(Default)]
    pub struct Tracker {
        map: Mutex<HashMap<NonNull<u8>, (usize, Layout)>>,
    }

    #[cfg(test)]
    unsafe impl CallbackRef for Tracker {
        #[track_caller]
        fn after_alloc(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
            if let Ok(ptr) = result {
                self.map
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(ptr.as_non_null_ptr(), (ptr.len(), layout));
            }
        }
        #[track_caller]
        fn after_alloc_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
            self.after_alloc(layout, result)
        }
        #[track_caller]
        fn after_alloc_all(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
            self.after_alloc(layout, result)
        }
        #[track_caller]
        fn after_alloc_all_zeroed(&self, layout: Layout, result: Result<NonNull<[u8]>, AllocErr>) {
            self.after_alloc(layout, result)
        }

        #[track_caller]
        fn before_grow(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
            assert!(
                new_size >= layout.size(),
                "`new_size` must be greater than or equal to `layout.size()`, expected {} >= {}",
                new_size,
                layout.size()
            );
            Layout::from_size_align(new_size, layout.align()).unwrap_or_else(|_| {
                panic!(
                    "`new_size`, when rounded up to the nearest multiple of `layout.align()`, \
                     must not overflow (i.e., the rounded value must be less than or equal to \
                     `usize::MAX`), expected {} to be rounded up to the nearest multiple of {}",
                    new_size,
                    layout.align()
                )
            });
            self.before_dealloc(ptr, layout)
        }

        #[track_caller]
        fn after_grow(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            assert!(
                new_size >= layout.size(),
                "`new_size` must be greater than or equal to `layout.size()`, expected {} >= {}",
                new_size,
                layout.size()
            );
            let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
            if result.is_ok() {
                self.after_dealloc(ptr, layout);
                self.after_alloc(new_layout, result);
            }
        }

        #[track_caller]
        fn before_grow_zeroed(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
            self.before_grow(ptr, layout, new_size)
        }

        #[track_caller]
        fn after_grow_zeroed(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            self.after_grow(ptr, layout, new_size, result)
        }

        #[track_caller]
        fn before_grow_in_place(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
            self.before_grow(ptr, layout, new_size)
        }

        #[track_caller]
        fn after_grow_in_place(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            result: Result<usize, AllocErr>,
        ) {
            self.after_grow(
                ptr,
                layout,
                new_size,
                result.map(|len| NonNull::slice_from_raw_parts(ptr, len)),
            )
        }

        #[track_caller]
        fn before_grow_in_place_zeroed(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
            self.before_grow(ptr, layout, new_size)
        }

        #[track_caller]
        fn after_grow_in_place_zeroed(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            result: Result<usize, AllocErr>,
        ) {
            self.after_grow(
                ptr,
                layout,
                new_size,
                result.map(|len| NonNull::slice_from_raw_parts(ptr, len)),
            )
        }

        #[track_caller]
        fn before_shrink(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
            assert!(
                new_size <= layout.size(),
                "`new_size` must be smaller than or equal to `layout.size()`, expected {} <= {}",
                new_size,
                layout.size()
            );
            self.before_dealloc(ptr, layout);
        }

        #[track_caller]
        fn after_shrink(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            result: Result<NonNull<[u8]>, AllocErr>,
        ) {
            assert!(
                new_size <= layout.size(),
                "`new_size` must be smaller than or equal to `layout.size()`, expected {} <= {}",
                new_size,
                layout.size()
            );
            let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
            if result.is_ok() {
                self.after_dealloc(ptr, layout);
                self.after_alloc(new_layout, result);
            }
        }

        #[track_caller]
        fn before_shrink_in_place(&self, ptr: NonNull<u8>, layout: Layout, new_size: usize) {
            self.before_shrink(ptr, layout, new_size)
        }

        #[track_caller]
        fn after_shrink_in_place(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            result: Result<usize, AllocErr>,
        ) {
            self.after_shrink(
                ptr,
                layout,
                new_size,
                result.map(|len| NonNull::slice_from_raw_parts(ptr, len)),
            )
        }

        #[track_caller]
        fn before_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
            let lock = self.map.lock().unwrap_or_else(PoisonError::into_inner);
            let (size, old_layout) = lock.get(&ptr).expect(
                "`ptr` must denote a block of memory currently allocated via this allocator",
            );
            assert_eq!(
                layout.align(),
                old_layout.align(),
                "`layout` must fit that block of memory. Expected alignment of {}, got {}",
                old_layout.align(),
                layout.align()
            );
            if layout.size() < old_layout.size() || layout.size() > *size {
                if *size == old_layout.size() {
                    panic!(
                        "`layout` must fit that block of memory. Expected size of {}, got {}",
                        old_layout.size(),
                        layout.size()
                    )
                } else {
                    panic!(
                        "`layout` must fit that block of memory. Expected size between {}..={}, \
                         got {}",
                        old_layout.size(),
                        size,
                        layout.size()
                    )
                }
            }
        }
    }

    #[test]
    #[should_panic = "`new_size` must be greater than or equal to `layout.size()`"]
    fn tracker_grow_size_greater_layout() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        let _ = unsafe { alloc.grow(memory.as_non_null_ptr(), Layout::new::<[u8; 4]>(), 2) };
    }

    #[test]
    #[should_panic = "`new_size`, when rounded up to the nearest multiple of `layout.align()`"]
    fn tracker_grow_size_rounded_up() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u64; 4]>())
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                memory.as_non_null_ptr(),
                Layout::new::<[u64; 4]>(),
                usize::MAX,
            )
        };
    }

    #[test]
    #[should_panic = "`layout` must fit that block of memory"]
    fn tracker_grow_layout_size_exact() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        let _ = unsafe { alloc.grow(memory.as_non_null_ptr(), Layout::new::<[u8; 2]>(), 10) };
    }

    #[test]
    #[should_panic = "`layout` must fit that block of memory"]
    fn tracker_grow_layout_size_range() {
        let mut alloc = tracker(Chunk::<System, 32>::default());
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        let _ = unsafe { alloc.grow(memory.as_non_null_ptr(), Layout::new::<[u8; 2]>(), 10) };
    }

    #[test]
    #[should_panic = "`layout` must fit that block of memory"]
    fn tracker_grow_layout_align() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        let _ = unsafe { alloc.grow(memory.as_non_null_ptr(), Layout::new::<[u16; 2]>(), 10) };
    }

    #[test]
    #[should_panic = "`ptr` must denote a block of memory currently allocated via this allocator"]
    fn tracker_grow_ptr() {
        let mut alloc = tracker(System);
        alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        let _ = unsafe { alloc.grow(NonNull::dangling(), Layout::new::<[u8; 4]>(), 10) };
    }
}
