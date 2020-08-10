use crate::{
    helper::{AllocInit, ReallocPlacement},
    unlikely,
    AllocAll,
    Owns,
    ReallocInPlace,
};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
    fmt,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

/// Allocator over an user-defined region of memory.
///
/// ## Examples
///
/// ```rust
/// #![feature(allocator_api)]
///
/// use alloc_compose::{Owns, Region};
/// use core::{
///     alloc::{AllocRef, Layout},
///     mem::MaybeUninit,
/// };
///
/// let mut data = [MaybeUninit::new(0); 64];
/// let mut region = Region::new(&mut data);
///
/// let memory = region.alloc(Layout::new::<u32>())?;
/// assert!(region.owns(memory));
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
/// It's possible to deallocate the latest memory block allocated:
///
/// ```rust
/// #![feature(slice_ptr_get)]
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Owns, Region};
/// # use core::{alloc::{AllocRef, Layout}, mem::MaybeUninit};
/// # let mut data = [MaybeUninit::new(0); 64];
/// # let mut region = Region::new(&mut data);
/// # let memory = region.alloc(Layout::new::<u32>())?;
///
/// unsafe { region.dealloc(memory.as_non_null_ptr(), Layout::new::<u32>()) };
/// assert!(!region.owns(memory));
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
pub struct Region<'a> {
    data: &'a mut [MaybeUninit<u8>],
    ptr: usize,
}

impl<'a> Region<'a> {
    #[inline]
    pub fn new(data: &'a mut [MaybeUninit<u8>]) -> Self {
        let ptr = data.as_ptr() as usize + data.len();
        let region = Self { data, ptr };
        debug_assert!(region.is_empty());
        region
    }

    /// Checks if `memory` is the latest block, which was allocated.
    /// For those blocks, it's possible to deallocate them.
    #[inline]
    pub fn is_last_block(&self, memory: NonNull<[u8]>) -> bool {
        self.ptr == memory.as_mut_ptr() as usize
    }

    fn start(&self) -> usize {
        self.data.as_ptr() as usize
    }
    fn end(&self) -> usize {
        self.data.as_ptr() as usize + self.data.len()
    }

    fn create_block(&mut self, ptr: usize) -> NonNull<[u8]> {
        let raw_ptr = unsafe { NonNull::new_unchecked(ptr as *mut u8) };
        let memory = NonNull::slice_from_raw_parts(raw_ptr, self.ptr - ptr);
        self.ptr = ptr;
        memory
    }
}

impl fmt::Debug for Region<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Region")
            .field("capacity", &self.capacity())
            .field("capacity_left", &self.capacity_left())
            .finish()
    }
}

impl Region<'_> {
    #[inline]
    #[track_caller]
    fn alloc_impl(
        &mut self,
        layout: Layout,
        init: AllocInit,
        all: bool,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let aligned_ptr = if all {
            let new_ptr = self.data.as_ptr() as usize;
            let aligned_ptr =
                new_ptr.checked_add(layout.align() - 1).ok_or(AllocErr)? & !(layout.align() - 1);

            if unlikely(aligned_ptr + layout.size() >= self.ptr) {
                return Err(AllocErr);
            }

            aligned_ptr
        } else {
            let new_ptr = self.ptr.checked_sub(layout.size()).ok_or(AllocErr)?;
            let aligned_ptr = new_ptr & !(layout.align() - 1);

            if unlikely(aligned_ptr < self.start()) {
                return Err(AllocErr);
            }

            aligned_ptr
        };

        let ptr = self.create_block(aligned_ptr);
        unsafe { init.init(ptr) };
        Ok(ptr)
    }

    #[inline]
    #[track_caller]
    unsafe fn grow_impl(
        &mut self,
        old_ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        crate::check_grow_precondition(old_ptr, old_layout, new_size);

        let old_memory = NonNull::slice_from_raw_parts(old_ptr, old_layout.size());
        debug_assert!(
            self.owns(old_memory),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );

        let old_size = old_layout.size();
        if old_size == new_size {
            Ok(old_memory)
        } else if self.is_last_block(old_memory) {
            let additional = new_size - old_size;
            let new_ptr = self.ptr.checked_sub(additional).ok_or(AllocErr)?;
            let aligned_ptr = new_ptr & !(old_layout.align() - 1);

            if unlikely(aligned_ptr < self.start()) {
                return Err(AllocErr);
            }

            let memory = NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(aligned_ptr as *mut u8),
                self.ptr + old_size - aligned_ptr,
            );
            self.ptr = aligned_ptr;
            ptr::copy(old_ptr.as_ptr(), memory.as_mut_ptr(), old_size);

            init.init_offset(memory, old_size);

            Ok(memory)
        } else if placement == ReallocPlacement::InPlace {
            Err(AllocErr)
        } else {
            let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());
            let new_memory = self.alloc_impl(new_layout, init, false)?;
            ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_memory.as_mut_ptr(), old_size);
            Ok(new_memory)
        }
    }

    #[inline]
    #[track_caller]
    unsafe fn shrink_impl(
        &mut self,
        old_ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        crate::check_shrink_precondition(old_ptr, old_layout, new_size);

        let old_size = old_layout.size();
        let old_memory = NonNull::slice_from_raw_parts(old_ptr, old_size);
        debug_assert!(
            self.owns(old_memory),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );

        if old_size == new_size {
            Ok(old_memory)
        } else if self.is_last_block(old_memory) {
            let new_ptr = self.ptr + old_size - new_size;
            let aligned_ptr = new_ptr & !(old_layout.align() - 1);

            let new_memory = NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(aligned_ptr as *mut u8),
                self.ptr + old_size - aligned_ptr,
            );
            ptr::copy(old_ptr.as_ptr(), new_memory.as_mut_ptr(), new_size);
            self.ptr = aligned_ptr;
            Ok(new_memory)
        } else if placement == ReallocPlacement::InPlace {
            Err(AllocErr)
        } else {
            Ok(old_memory)
        }
    }
}

unsafe impl AllocRef for Region<'_> {
    #[track_caller]
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.alloc_impl(layout, AllocInit::Uninitialized, false)
    }

    #[track_caller]
    fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.alloc_impl(layout, AllocInit::Zeroed, false)
    }

    #[track_caller]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        crate::check_dealloc_precondition(ptr, layout);

        let memory = NonNull::slice_from_raw_parts(ptr, layout.size());
        debug_assert!(
            self.owns(memory),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );

        if self.is_last_block(memory) {
            self.ptr += layout.size()
        }
    }

    #[track_caller]
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        self.grow_impl(
            ptr,
            layout,
            new_size,
            ReallocPlacement::MayMove,
            AllocInit::Uninitialized,
        )
    }

    #[track_caller]
    unsafe fn grow_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        self.grow_impl(
            ptr,
            layout,
            new_size,
            ReallocPlacement::MayMove,
            AllocInit::Zeroed,
        )
    }

    #[track_caller]
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        self.shrink_impl(ptr, layout, new_size, ReallocPlacement::MayMove)
    }
}

unsafe impl AllocAll for Region<'_> {
    #[track_caller]
    fn alloc_all(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.alloc_impl(layout, AllocInit::Uninitialized, true)
    }

    #[track_caller]
    fn alloc_all_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocErr> {
        self.alloc_impl(layout, AllocInit::Zeroed, true)
    }

    #[track_caller]
    fn dealloc_all(&mut self) {
        self.ptr = self.end();
        debug_assert!(self.is_empty());
    }

    #[track_caller]
    fn capacity(&self) -> usize {
        self.data.len()
    }

    #[track_caller]
    fn capacity_left(&self) -> usize {
        self.ptr - self.start()
    }
}

unsafe impl ReallocInPlace for Region<'_> {
    #[track_caller]
    unsafe fn grow_in_place(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, AllocErr> {
        self.grow_impl(
            ptr,
            layout,
            new_size,
            ReallocPlacement::InPlace,
            AllocInit::Uninitialized,
        )
        .map(NonNull::len)
    }

    #[track_caller]
    unsafe fn grow_in_place_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, AllocErr> {
        self.grow_impl(
            ptr,
            layout,
            new_size,
            ReallocPlacement::InPlace,
            AllocInit::Zeroed,
        )
        .map(NonNull::len)
    }

    #[track_caller]
    unsafe fn shrink_in_place(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, AllocErr> {
        self.shrink_impl(ptr, layout, new_size, ReallocPlacement::InPlace)
            .map(NonNull::len)
    }
}

impl Owns for Region<'_> {
    fn owns(&self, memory: NonNull<[u8]>) -> bool {
        let ptr = memory.as_mut_ptr() as usize;
        ptr >= self.ptr && ptr + memory.len() <= self.end()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]
    use super::*;
    use std::{
        alloc::{Global, Layout},
        slice,
    };

    #[test]
    fn alloc_zero() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert!(region.is_empty());

        region
            .alloc(Layout::new::<[u8; 0]>())
            .expect("Could not allocated 0 bytes");
        assert!(region.is_empty());

        unsafe {
            assert_eq!(MaybeUninit::slice_get_ref(&data), &[1; 32][..]);
        }
    }

    #[test]
    fn alloc_zeroed() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert!(region.is_empty());

        region
            .alloc_zeroed(Layout::new::<[u8; 32]>())
            .expect("Could not allocated 32 bytes");
        assert!(!region.is_empty());

        unsafe {
            assert_eq!(MaybeUninit::slice_get_ref(&data), &[0; 32][..]);
        }
    }

    #[test]
    fn alloc_all() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert!(region.is_empty());

        let ptr = region
            .alloc(Layout::new::<u8>())
            .expect("Could not allocated 1 byte");
        assert_eq!(ptr.len(), 1);
        assert_eq!(region.capacity_left(), 31);

        let ptr = region
            .alloc_all_zeroed(Layout::new::<[u8; 4]>())
            .expect("Could not allocated 4 bytes");
        assert_eq!(ptr.len(), 31);
        assert!(region.is_full());

        region.dealloc_all();
        assert!(region.is_empty());

        region
            .alloc(Layout::new::<[u8; 16]>())
            .expect("Could not allocate 16 bytes");
        region
            .alloc_all(Layout::new::<[u8; 17]>())
            .expect_err("Could allocate more than 32 bytes");
    }

    #[test]
    fn alloc_small() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert_eq!(region.capacity(), region.capacity_left());

        region
            .alloc_zeroed(Layout::new::<[u8; 16]>())
            .expect("Could not allocated 16 bytes");
        assert_eq!(region.capacity_left(), 16);

        unsafe {
            assert_eq!(MaybeUninit::slice_get_ref(&data[0..16]), &[1; 16][..]);
            assert_eq!(MaybeUninit::slice_get_ref(&data[16..]), &[0; 16][..]);
        }
    }

    #[test]
    fn alloc_uninitialzed() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        region
            .alloc(Layout::new::<[u8; 32]>())
            .expect("Could not allocated 32 bytes");
        assert_eq!(region.capacity_left(), 0);

        unsafe {
            assert_eq!(MaybeUninit::slice_get_ref(&data), &[1; 32][..]);
        }
    }

    #[test]
    fn alloc_fail() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        region
            .alloc(Layout::new::<[u8; 33]>())
            .expect_err("Could allocate 33 bytes");
    }

    #[test]
    fn alloc_aligned() {
        let memory = Global
            .alloc(Layout::from_size_align(1024, 64).expect("Invalid layout"))
            .expect("Could not allocate 1024 Bytes");
        assert_eq!(memory.as_mut_ptr() as usize % 64, 0);
        let data = unsafe {
            slice::from_raw_parts_mut(memory.as_non_null_ptr().cast().as_ptr(), memory.len())
        };
        let mut region = Region::new(data);

        region
            .alloc(Layout::from_size_align(5, 1).expect("Invalid layout"))
            .expect("Could not allocate 5 Bytes");

        let memory = region
            .alloc(Layout::from_size_align(16, 16).expect("Invalid layout"))
            .expect("Could not allocate 16 Bytes");
        assert_eq!(memory.as_mut_ptr() as usize % 16, 0);
    }

    #[test]
    fn dealloc() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);
        let layout = Layout::from_size_align(8, 1).expect("Invalid layout");

        let memory = region.alloc(layout).expect("Could not allocate 8 bytes");
        assert!(region.owns(memory));
        assert_eq!(region.capacity_left(), 24);

        unsafe {
            region.dealloc(memory.as_non_null_ptr(), layout);
        }
        assert_eq!(region.capacity_left(), 32);
        assert!(!region.owns(memory));

        let memory = region.alloc(layout).expect("Could not allocate 8 bytes");
        assert!(region.owns(memory));
        region.alloc(layout).expect("Could not allocate 8 bytes");
        assert!(region.owns(memory));
        assert_eq!(memory.len(), 8);
        assert_eq!(region.capacity_left(), 16);

        unsafe {
            region.dealloc(memory.as_non_null_ptr(), layout);
        }
        // It is not possible to deallocate memory that was not allocated last.
        assert!(region.owns(memory));
        assert_eq!(region.capacity_left(), 16);
    }

    #[test]
    fn realloc() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);
        let layout = Layout::from_size_align(8, 1).expect("Invalid layout");

        let memory = region.alloc(layout).expect("Could not allocate 8 bytes");
        assert_eq!(memory.len(), 8);
        assert_eq!(region.capacity_left(), 24);

        region.alloc(layout).expect("Could not allocate 8 bytes");
        assert_eq!(region.capacity_left(), 16);

        let memory = unsafe {
            region
                .grow(memory.as_non_null_ptr(), layout, 16)
                .expect("Could not grow to 16 bytes")
        };
        assert_eq!(memory.len(), 16);
        assert_eq!(region.capacity_left(), 0);

        region.dealloc_all();
        let memory = region
            .alloc_zeroed(Layout::new::<[u8; 16]>())
            .expect("Could not allocate 16 bytes");
        region
            .alloc(Layout::new::<[u8; 8]>())
            .expect("Could not allocate 16 bytes");

        unsafe {
            region
                .shrink(memory.as_non_null_ptr(), Layout::new::<[u8; 16]>(), 8)
                .expect("Could not shrink to 8 bytes");
        }
    }

    #[test]
    fn debug() {
        let test_output = |region: &Region| {
            assert_eq!(
                format!("{:?}", region),
                format!(
                    "Region {{ capacity: {}, capacity_left: {} }}",
                    region.capacity(),
                    region.capacity_left()
                )
            )
        };

        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);
        test_output(&region);

        region
            .alloc(Layout::new::<[u8; 16]>())
            .expect("Could not allocate 16 bytes");
        test_output(&region);

        region
            .alloc(Layout::new::<[u8; 16]>())
            .expect("Could not allocate 16 bytes");
        test_output(&region);

        region.dealloc_all();
        test_output(&region);
    }
}
