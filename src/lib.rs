#![cfg_attr(not(test), no_std)]
#![cfg_attr(doc, feature(doc_cfg, external_doc))]
#![cfg_attr(doc, doc(include = "../README.md"))]
#![feature(allocator_api)]

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

mod callback_ref;
mod fallback_alloc;
mod null_alloc;
mod proxy;
mod region;
mod segregate_alloc;
pub mod stats;

use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::{self, NonNull},
};

pub use self::{
    callback_ref::CallbackRef,
    fallback_alloc::FallbackAlloc,
    null_alloc::NullAlloc,
    proxy::Proxy,
    region::Region,
    segregate_alloc::SegregateAlloc,
};

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
