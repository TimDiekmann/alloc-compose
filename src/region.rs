use crate::{unlikely, AllocAll, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
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
/// use core::alloc::{AllocInit, AllocRef, Layout};
/// use core::mem::MaybeUninit;
///
/// let mut data = [MaybeUninit::new(0); 64];
/// let mut region = Region::new(&mut data);
///
/// let memory = region.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
/// assert!(region.owns(memory));
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
/// It's possible to deallocate the latest memory block allocated:
///
/// ```rust
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Owns, Region};
/// # use core::{alloc::{AllocInit, AllocRef, Layout}, mem::MaybeUninit};
/// # let mut data = [MaybeUninit::new(0); 64];
/// # let mut region = Region::new(&mut data);
/// # let memory = region.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
/// unsafe { region.dealloc(memory.ptr, Layout::new::<u32>()) };
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
    pub fn is_last_block(&self, memory: MemoryBlock) -> bool {
        self.ptr == memory.ptr.as_ptr() as usize
    }

    fn start(&self) -> usize {
        self.data.as_ptr() as usize
    }
    fn end(&self) -> usize {
        self.data.as_ptr() as usize + self.data.len()
    }

    fn create_block(&mut self, ptr: usize) -> Result<MemoryBlock, AllocErr> {
        let memory = MemoryBlock {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut u8) },
            size: self.ptr - ptr,
        };
        self.ptr = ptr;
        Ok(memory)
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

unsafe impl AllocRef for Region<'_> {
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        let new_ptr = self.ptr.checked_sub(layout.size()).ok_or(AllocErr)?;
        let aligned_ptr = new_ptr & !(layout.align() - 1);

        if unlikely(aligned_ptr < self.start()) {
            return Err(AllocErr);
        }

        let memory = self.create_block(aligned_ptr)?;
        unsafe { init.init(memory) };
        Ok(memory)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let memory = MemoryBlock {
            ptr,
            size: layout.size(),
        };
        debug_assert!(
            self.owns(memory),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );
        if self.is_last_block(memory) {
            self.ptr += layout.size()
        }
    }

    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        let memory = MemoryBlock {
            ptr,
            size: layout.size(),
        };
        debug_assert!(
            self.owns(memory),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );
        let old_size = layout.size();
        debug_assert!(
            new_size >= old_size,
            "`new_size` must be greater than or equal to `layout.size()`"
        );

        if old_size == new_size {
            Ok(memory)
        } else if placement == ReallocPlacement::InPlace {
            Err(AllocErr)
        } else if self.is_last_block(memory) {
            let additional = new_size - old_size;
            let new_ptr = self.ptr.checked_sub(additional).ok_or(AllocErr)?;
            let aligned_ptr = new_ptr & !(layout.align() - 1);

            if unlikely(aligned_ptr < self.start()) {
                return Err(AllocErr);
            }

            let memory = MemoryBlock {
                ptr: NonNull::new_unchecked(aligned_ptr as *mut u8),
                size: self.ptr + layout.size() - aligned_ptr,
            };
            self.ptr = aligned_ptr;
            ptr::copy(ptr.as_ptr(), memory.ptr.as_ptr(), old_size);
            init.init_offset(memory, old_size);

            Ok(memory)
        } else {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_memory = self.alloc(new_layout, init)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_memory.ptr.as_ptr(), old_size);
            Ok(new_memory)
        }
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        let old_size = layout.size();
        let memory = MemoryBlock {
            ptr,
            size: old_size,
        };
        debug_assert!(
            self.owns(memory),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );
        debug_assert!(
            new_size <= old_size,
            "`new_size` must be smaller than or equal to `layout.size()`"
        );

        if old_size == new_size {
            Ok(memory)
        } else if self.is_last_block(memory) {
            match placement {
                ReallocPlacement::MayMove => {
                    let new_ptr = self.ptr + old_size - new_size;
                    let aligned_ptr = new_ptr & !(layout.align() - 1);

                    let memory = MemoryBlock {
                        ptr: NonNull::new_unchecked(aligned_ptr as *mut u8),
                        size: self.ptr + old_size - aligned_ptr,
                    };
                    ptr::copy(ptr.as_ptr(), memory.ptr.as_ptr(), new_size);
                    self.ptr = aligned_ptr;
                    Ok(memory)
                }
                ReallocPlacement::InPlace => Err(AllocErr),
            }
        } else {
            Ok(memory)
        }
    }
}

impl AllocAll for Region<'_> {
    fn alloc_all(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        let new_ptr = self.data.as_ptr() as usize;
        let aligned_ptr =
            new_ptr.checked_add(layout.align() - 1).ok_or(AllocErr)? & !(layout.align() - 1);

        if unlikely(aligned_ptr + layout.size() >= self.ptr) {
            return Err(AllocErr);
        }

        let memory = self.create_block(aligned_ptr)?;
        unsafe { init.init(memory) };
        Ok(memory)
    }

    fn dealloc_all(&mut self) {
        self.ptr = self.end();
        debug_assert!(self.is_empty());
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn capacity_left(&self) -> usize {
        self.ptr - self.start()
    }
}

impl Owns for Region<'_> {
    fn owns(&self, memory: MemoryBlock) -> bool {
        let ptr = memory.ptr.as_ptr() as usize;
        ptr >= self.ptr && ptr + memory.size <= self.end()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]
    use super::*;
    use crate::helper::AsSlice;
    use std::alloc::{Global, Layout};

    #[test]
    fn alloc_zero() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert!(region.is_empty());

        region
            .alloc(Layout::new::<[u8; 0]>(), AllocInit::Uninitialized)
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
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Zeroed)
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
            .alloc(Layout::new::<u8>(), AllocInit::Uninitialized)
            .expect("Could not allocated 1 byte");
        assert_eq!(ptr.size, 1);
        assert_eq!(region.capacity_left(), 31);

        let ptr = region
            .alloc_all(Layout::new::<[u8; 4]>(), AllocInit::Uninitialized)
            .expect("Could not allocated 4 bytes");
        assert_eq!(ptr.size, 31);
        assert!(region.is_full());

        region.dealloc_all();
        assert!(region.is_empty());

        region
            .alloc(Layout::new::<[u8; 16]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 16 bytes");
        region
            .alloc_all(Layout::new::<[u8; 17]>(), AllocInit::Uninitialized)
            .expect_err("Could allocate more than 32 bytes");
    }

    #[test]
    fn alloc_small() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert_eq!(region.capacity(), region.capacity_left());

        region
            .alloc(Layout::new::<[u8; 16]>(), AllocInit::Zeroed)
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
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
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
            .alloc(Layout::new::<[u8; 33]>(), AllocInit::Uninitialized)
            .expect_err("Could allocate 33 bytes");
    }

    #[test]
    fn alloc_aligned() {
        let memory = Global
            .alloc(
                Layout::from_size_align(1024, 64).expect("Invalid layout"),
                AllocInit::Uninitialized,
            )
            .expect("Could not allocate 1024 Bytes");
        assert_eq!(memory.ptr.as_ptr() as usize % 64, 0);
        let data = unsafe { memory.as_slice_mut() };
        let mut region = Region::new(data);

        region
            .alloc(
                Layout::from_size_align(5, 1).expect("Invalid layout"),
                AllocInit::Uninitialized,
            )
            .expect("Could not allocate 5 Bytes");

        let memory = region
            .alloc(
                Layout::from_size_align(16, 16).expect("Invalid layout"),
                AllocInit::Uninitialized,
            )
            .expect("Could not allocate 16 Bytes");
        assert_eq!(memory.ptr.as_ptr() as usize % 16, 0);
    }

    #[test]
    fn dealloc() {
        let mut data = [MaybeUninit::new(1); 32];
        let mut region = Region::new(&mut data);
        let layout = Layout::from_size_align(8, 1).expect("Invalid layout");

        let memory = region
            .alloc(layout, AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        assert!(region.owns(memory));
        assert_eq!(region.capacity_left(), 24);

        unsafe {
            region.dealloc(memory.ptr, layout);
        }
        assert_eq!(region.capacity_left(), 32);
        assert!(!region.owns(memory));

        let memory = region
            .alloc(layout, AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        assert!(region.owns(memory));
        region
            .alloc(layout, AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        assert!(region.owns(memory));
        assert_eq!(memory.size, 8);
        assert_eq!(region.capacity_left(), 16);

        unsafe {
            region.dealloc(memory.ptr, layout);
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

        let memory = region
            .alloc(layout, AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        assert_eq!(memory.size, 8);
        assert_eq!(region.capacity_left(), 24);

        region
            .alloc(layout, AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        assert_eq!(region.capacity_left(), 16);

        let memory = unsafe {
            region
                .grow(
                    memory.ptr,
                    layout,
                    16,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 16 bytes")
        };
        assert_eq!(memory.size, 16);
        assert_eq!(region.capacity_left(), 0);

        region.dealloc_all();
        let memory = region
            .alloc(Layout::new::<[u8; 16]>(), AllocInit::Zeroed)
            .expect("Could not allocate 16 bytes");
        region
            .alloc(Layout::new::<[u8; 8]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 16 bytes");

        unsafe {
            region
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 16]>(),
                    8,
                    ReallocPlacement::MayMove,
                )
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
            .alloc(Layout::new::<[u8; 16]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 16 bytes");
        test_output(&region);

        region
            .alloc(Layout::new::<[u8; 16]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 16 bytes");
        test_output(&region);

        region.dealloc_all();
        test_output(&region);
    }
}
