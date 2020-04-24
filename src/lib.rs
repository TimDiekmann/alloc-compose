//! Composable allocator structures for plugging together more powerful allocators.
//!
//! `alloc-compose` relies on [`AllocRef`] as allocator trait. Until `AllocRef` has been
//! stabilized, this crate requires a nightly compiler.
//!
//! [`AllocRef`]: core::alloc::AllocRef
//!
//! The design of composable allocators is inspired by
//! [`std::allocator` Is to Allocation what `std::vector` Is to Vexation][vid] by Andrei
//! Alexandrescu and the [Phobos Standard Library][phobos] of the [D Programming Language][D].
//!
//! [vid]: https://www.youtube.com/watch?v=LIb3L4vKZ7U
//! [phobos]: https://github.com/dlang/phobos
//! [D]: https://dlang.org/

#![no_std]
#![feature(allocator_api)]

mod fallback_alloc;
mod null_alloc;
mod region;
mod segregate_alloc;

use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::{self, NonNull},
};

pub use self::{
    fallback_alloc::FallbackAlloc,
    null_alloc::NullAlloc,
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
