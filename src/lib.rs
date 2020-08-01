#![cfg_attr(not(test), no_std)]
#![cfg_attr(doc, feature(doc_cfg, external_doc))]
#![cfg_attr(feature = "intrinsics", feature(core_intrinsics))]
#![cfg_attr(doc, doc(include = "../README.md"))]
#![feature(
    allocator_api,
    alloc_layout_extra,
    const_checked_int_methods,
    const_alloc_layout,
    const_fn,
    const_generics,
    const_panic
)]
#![allow(incomplete_features, clippy::must_use_candidate)]

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

pub mod stats;

mod affix;
mod callback_ref;
mod chunk;
mod fallback;
mod null;
mod proxy;
mod region;
mod segregate;

use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::{self, NonNull},
};

pub use self::{
    affix::Affix,
    callback_ref::CallbackRef,
    chunk::Chunk,
    fallback::Fallback,
    null::Null,
    proxy::Proxy,
    region::Region,
    segregate::Segregate,
};

#[cfg(feature = "intrinsics")]
mod intrinsics {
    pub use core::intrinsics::unlikely;
}

#[cfg(not(feature = "intrinsics"))]
mod intrinsics {
    #[inline(always)]
    pub fn unlikely(b: bool) -> bool {
        b
    }
}

use crate::intrinsics::*;

pub trait AllocAll {
    /// Attempts to allocate all of the memory the allocator can provide.
    ///
    /// If the allocator is currently not managing any memory, then it returns all the memory
    /// available to the allocator. Subsequent calls should not suceed.
    ///
    /// On success, returns a [`MemoryBlock`][] meeting the size and alignment guarantees of `layout`.
    ///
    /// The returned block is at least as large as specified by `layout.size()` and is
    /// initialized as specified by [`init`], all the way up to the returned size of the block.
    ///
    /// [`init`]: AllocInit
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout` does not meet
    /// allocator's size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    fn alloc_all(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr>;

    /// Deallocates all the memory the allocator had allocated.
    fn dealloc_all(&mut self);

    /// Returns the total capacity available in this allocator.
    fn capacity(&self) -> usize;

    /// Returns the free capacity left for allocating.
    fn capacity_left(&self) -> usize;

    /// Returns if the allocator is currently not holding memory.
    fn is_empty(&self) -> bool {
        self.capacity() == self.capacity_left()
    }

    /// Returns if the allocator has no more capacity left.
    fn is_full(&self) -> bool {
        self.capacity_left() == 0
    }
}

/// Trait to determine if a given `MemoryBlock` is owned by an allocator.
pub trait Owns {
    /// Returns if the allocator *owns* the passed `MemoryBlock`.
    fn owns(&self, memory: MemoryBlock) -> bool;
}

unsafe fn grow<A1: AllocRef, A2: AllocRef>(
    a1: &mut A1,
    a2: &mut A2,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
    placement: ReallocPlacement,
    init: AllocInit,
) -> Result<MemoryBlock, AllocErr> {
    if placement == ReallocPlacement::MayMove {
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let new_memory = a2.alloc(new_layout, init)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_memory.ptr.as_ptr(), layout.size());
        a1.dealloc(ptr, layout);
        Ok(new_memory)
    } else {
        Err(AllocErr)
    }
}

unsafe fn shrink<A1: AllocRef, A2: AllocRef>(
    a1: &mut A1,
    a2: &mut A2,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
    placement: ReallocPlacement,
) -> Result<MemoryBlock, AllocErr> {
    if placement == ReallocPlacement::MayMove {
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let new_memory = a2.alloc(new_layout, AllocInit::Uninitialized)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_memory.ptr.as_ptr(), new_memory.size);
        a1.dealloc(ptr, layout);
        Ok(new_memory)
    } else {
        Err(AllocErr)
    }
}

#[cfg(test)]
pub(crate) mod helper {
    use crate::{CallbackRef, Chunk, Proxy};
    use std::{
        alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement, System},
        collections::HashMap,
        ptr::NonNull,
        slice,
        sync::{Mutex, PoisonError},
    };

    #[derive(Default)]
    pub struct Tracker {
        map: Mutex<HashMap<NonNull<u8>, (usize, Layout)>>,
    }

    unsafe impl CallbackRef for Tracker {
        fn after_alloc(
            &self,
            layout: Layout,
            _init: AllocInit,
            result: Result<MemoryBlock, AllocErr>,
        ) {
            if let Ok(memory) = result {
                self.map
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(memory.ptr, (memory.size, layout));
            }
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

        fn after_dealloc(&self, ptr: NonNull<u8>, _layout: Layout) {
            self.map
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&ptr)
                .unwrap();
        }

        #[track_caller]
        fn before_grow(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            _placement: ReallocPlacement,
            _init: AllocInit,
        ) {
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
            _placement: ReallocPlacement,
            init: AllocInit,
            result: Result<MemoryBlock, AllocErr>,
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
                self.after_alloc(new_layout, init, result);
            }
        }

        #[track_caller]
        fn before_shrink(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            _placement: ReallocPlacement,
        ) {
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
            _placement: ReallocPlacement,
            result: Result<MemoryBlock, AllocErr>,
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
                self.after_alloc(new_layout, AllocInit::Uninitialized, result);
            }
        }
    }

    pub fn tracker<A: AllocRef>(alloc: A) -> Proxy<A, impl CallbackRef> {
        Proxy {
            alloc,
            callbacks: Tracker::default(),
        }
    }

    pub trait AsSlice {
        unsafe fn as_slice<'a>(self) -> &'a [u8];
        unsafe fn as_slice_mut<'a>(self) -> &'a mut [u8];
    }

    impl AsSlice for MemoryBlock {
        unsafe fn as_slice<'a>(self) -> &'a [u8] {
            slice::from_raw_parts(self.ptr.as_ptr(), self.size)
        }
        unsafe fn as_slice_mut<'a>(self) -> &'a mut [u8] {
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size)
        }
    }

    #[test]
    #[should_panic = "`new_size` must be greater than or equal to `layout.size()`"]
    fn tracker_grow_size_greater_layout() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                memory.ptr,
                Layout::new::<[u8; 4]>(),
                2,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            )
        };
    }

    #[test]
    #[should_panic = "`new_size`, when rounded up to the nearest multiple of `layout.align()`"]
    fn tracker_grow_size_rounded_up() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u64; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                memory.ptr,
                Layout::new::<[u64; 4]>(),
                usize::MAX,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            )
        };
    }

    #[test]
    #[should_panic = "`layout` must fit that block of memory"]
    fn tracker_grow_layout_size_exact() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                memory.ptr,
                Layout::new::<[u8; 2]>(),
                10,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            )
        };
    }

    #[test]
    #[should_panic = "`layout` must fit that block of memory"]
    fn tracker_grow_layout_size_range() {
        let mut alloc = tracker(Chunk::<System, 32>::default());
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                memory.ptr,
                Layout::new::<[u8; 2]>(),
                10,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            )
        };
    }

    #[test]
    #[should_panic = "`layout` must fit that block of memory"]
    fn tracker_grow_layout_align() {
        let mut alloc = tracker(System);
        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                memory.ptr,
                Layout::new::<[u16; 2]>(),
                10,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            )
        };
    }

    #[test]
    #[should_panic = "`ptr` must denote a block of memory currently allocated via this allocator"]
    fn tracker_grow_ptr() {
        let mut alloc = tracker(System);
        alloc
            .alloc(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let _ = unsafe {
            alloc.grow(
                NonNull::dangling(),
                Layout::new::<[u8; 4]>(),
                10,
                ReallocPlacement::MayMove,
                AllocInit::Uninitialized,
            )
        };
    }
}
