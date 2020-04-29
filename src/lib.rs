// #![cfg_attr(not(test), no_std)]
#![cfg_attr(doc, feature(doc_cfg, external_doc))]
#![cfg_attr(doc, doc(include = "../README.md"))]
#![feature(
    allocator_api,
    alloc_layout_extra,
    const_alloc_layout,
    const_fn,
    const_generics,
    const_if_match,
    const_panic,
    track_caller
)]
#![allow(incomplete_features)]

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

pub mod stats;

mod affix;
mod callback_ref;
mod chunk_alloc;
mod fallback_alloc;
mod memory_marker;
mod null_alloc;
mod proxy;
mod region;
mod segregate_alloc;

use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::{self, NonNull},
};

pub use self::{
    affix::Affix,
    callback_ref::CallbackRef,
    chunk_alloc::ChunkAlloc,
    fallback_alloc::FallbackAlloc,
    memory_marker::MemoryMarker,
    null_alloc::NullAlloc,
    proxy::Proxy,
    region::Region,
    segregate_alloc::SegregateAlloc,
};

type Result<T = MemoryBlock, E = AllocErr> = core::result::Result<T, E>;

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
) -> Result {
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
) -> Result {
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
    use crate::{CallbackRef, Proxy, Result};
    use std::{
        alloc::{AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
        collections::HashMap,
        ptr::NonNull,
        slice,
        sync::{Mutex, PoisonError},
    };

    #[derive(Default)]
    pub struct Tracker {
        map: Mutex<HashMap<NonNull<u8>, (usize, Layout)>>,
    }

    impl Tracker {
        fn insert(&self, memory: MemoryBlock, layout: Layout) {
            self.map
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(memory.ptr, (memory.size, layout));
        }

        #[track_caller]
        fn remove(&self, ptr: NonNull<u8>, layout: Layout) {
            let (size, old_layout) = self
                .map
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&ptr)
                .expect(
                    "dealloc: `ptr` must denote a block of memory currently allocated via this \
                     allocator",
                );
            assert_eq!(
                layout.align(),
                old_layout.align(),
                "dealloc: `layout` must fit that block of memory. Expected alignment of {}, got {}",
                old_layout.align(),
                layout.align()
            );
            if layout.size() < old_layout.size() || layout.size() > size {
                if size == old_layout.size() {
                    panic!(
                        "dealloc: `layout` must fit that block of memory. Expected size of {}, \
                         got {}",
                        old_layout.size(),
                        layout.size()
                    )
                } else {
                    panic!(
                        "dealloc: `layout` must fit that block of memory. Expected size between \
                         {}..={}, got {}",
                        old_layout.size(),
                        size,
                        layout.size()
                    )
                }
            }
        }
    }

    unsafe impl CallbackRef for Tracker {
        fn alloc(&self, layout: Layout, _init: AllocInit, result: Result) {
            if let Ok(memory) = result {
                self.insert(memory, layout)
            }
        }

        #[track_caller]
        fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
            self.remove(ptr, layout);
        }

        #[track_caller]
        fn grow(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            _placement: ReallocPlacement,
            _init: AllocInit,
            result: Result,
        ) {
            assert!(
                new_size >= layout.size(),
                "`new_size` must be greater than or equal to `layout.size()`, expected {} >= {}",
                new_size,
                layout.size()
            );
            let new_layout =
                Layout::from_size_align(new_size, layout.align()).unwrap_or_else(|_| {
                    panic!(
                        "`new_size`, when rounded up to the nearest multiple of `layout.align()`, \
                         must not overflow (i.e., the rounded value must be less than or equal to \
                         `usize::MAX`), expected {} to be rounded up to the nearest multiple of {}",
                        new_size,
                        layout.align()
                    )
                });
            if let Ok(memory) = result {
                self.remove(ptr, layout);
                self.insert(memory, new_layout);
            }
        }

        #[track_caller]
        fn shrink(
            &self,
            ptr: NonNull<u8>,
            layout: Layout,
            new_size: usize,
            _placement: ReallocPlacement,
            result: Result,
        ) {
            assert!(
                new_size <= layout.size(),
                "`new_size` must be smaller than or equal to `layout.size()`, expected {} <= {}",
                new_size,
                layout.size()
            );
            let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
            if let Ok(memory) = result {
                self.remove(ptr, layout);
                self.insert(memory, new_layout);
            }
        }

        fn owns(&self, _success: bool) {}
    }

    // impl Drop for Tracker {
    //     fn drop(&mut self) {
    //         let map = self.map.get_mut().unwrap_or_else(PoisonError::into_inner);
    //         if !map.is_empty() {
    //             let mut error = String::from("Not all allocations has been freed:");
    //             for (ptr, (size, layout)) in map {
    //                 if *size == layout.size() {
    //                     error.push_str(&format!(
    //                         "\n- {:?}: Layout {{ size: {}, align: {} }}",
    //                         ptr,
    //                         size,
    //                         layout.align()
    //                     ));
    //                 } else {
    //                     error.push_str(&format!(
    //                         "\n- {:?}: Layout {{ size: {}..={}, align: {} }}",
    //                         ptr,
    //                         layout.size(),
    //                         size,
    //                         layout.align()
    //                     ));
    //                 }
    //             }
    //             panic!("{}", error);
    //         }
    //     }
    // }

    pub fn tracker<A: AllocRef>(alloc: A) -> impl AllocRef {
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
}
