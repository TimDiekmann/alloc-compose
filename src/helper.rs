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
