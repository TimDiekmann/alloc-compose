use crate::{AllocAll, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    fmt,
    ptr,
    ptr::NonNull,
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
///
/// let mut data = [0; 64];
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
/// # use core::alloc::{AllocInit, AllocRef, Layout};
/// # let mut data = [0; 64];
/// # let mut region = Region::new(&mut data);
/// # let memory = region.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
/// unsafe { region.dealloc(memory.ptr, Layout::new::<u32>()) };
/// assert!(!region.owns(memory));
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
pub struct Region<'a> {
    data: &'a mut [u8],
    offset: usize,
}

impl<'a> Region<'a> {
    #[inline]
    pub fn new(data: &'a mut [u8]) -> Self {
        let current = data.as_ptr() as usize;
        Self {
            data,
            offset: current,
        }
    }

    /// Checks if `memory` is the latest block, which was allocated.
    /// For those blocks, it's possible to deallocate them or to grow
    /// or shrink them in place.
    #[inline]
    pub fn is_last_block(&self, memory: MemoryBlock) -> bool {
        memory.ptr.as_ptr() as usize + memory.size == self.offset as usize
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
        let offset = (self.offset as *mut u8).align_offset(layout.align());
        let current = self.offset.checked_add(offset).ok_or(AllocErr)?;

        let new = current.checked_add(layout.size()).ok_or(AllocErr)?;
        if new > self.data.as_ptr() as usize + self.data.len() {
            return Err(AllocErr);
        }

        self.offset = new;
        let memory = MemoryBlock {
            ptr: unsafe { NonNull::new_unchecked(current as *mut u8) },
            size: layout.size(),
        };
        unsafe { init.init(memory) };

        Ok(memory)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert!(
            self.owns(MemoryBlock {
                ptr,
                size: layout.size()
            }),
            "`ptr` must denote a block of memory currently allocated via this allocator"
        );
        if self.is_last_block(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.offset = ptr.as_ptr() as usize;
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
        let size = layout.size();
        debug_assert!(
            new_size >= size,
            "`new_size` must be greater than or equal to `layout.size()`"
        );

        if layout.size() == new_size {
            Ok(MemoryBlock {
                ptr,
                size: new_size,
            })
        } else if self.is_last_block(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            let start = ptr.as_ptr() as usize;
            let new = start.checked_add(new_size).ok_or(AllocErr)?;
            if new > self.data.as_ptr() as usize + self.data.len() {
                return Err(AllocErr);
            }
            self.offset = new;
            let new_memory = MemoryBlock {
                ptr,
                size: new_size,
            };
            init.init_offset(new_memory, layout.size());
            Ok(new_memory)
        } else if placement == ReallocPlacement::MayMove {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_memory = self.alloc(new_layout, init)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_memory.ptr.as_ptr(), size);
            Ok(new_memory)
        } else {
            Err(AllocErr)
        }
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        let size = layout.size();
        debug_assert!(
            new_size <= size,
            "`new_size` must be smaller than or equal to `layout.size()`"
        );

        if layout.size() == new_size {
            Ok(MemoryBlock {
                ptr,
                size: new_size,
            })
        } else if self.is_last_block(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.offset = ptr.as_ptr() as usize + new_size;
            Ok(MemoryBlock {
                ptr,
                size: new_size,
            })
        } else if placement == ReallocPlacement::MayMove {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_memory = self.alloc(new_layout, AllocInit::Uninitialized)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_memory.ptr.as_ptr(), new_size);
            Ok(new_memory)
        } else {
            Err(AllocErr)
        }
    }
}

impl AllocAll for Region<'_> {
    fn alloc_all(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        let offset = (self.offset as *mut u8).align_offset(layout.align());
        let current = self.offset.checked_add(offset).ok_or(AllocErr)?;

        let new = current.checked_add(layout.size()).ok_or(AllocErr)?;
        if new > self.data.as_ptr() as usize + self.data.len() {
            return Err(AllocErr);
        }

        let capacity_left = self.capacity_left();
        self.offset += capacity_left;
        let memory = MemoryBlock {
            ptr: unsafe { NonNull::new_unchecked(current as *mut u8) },
            size: capacity_left,
        };
        unsafe { init.init(memory) };

        Ok(memory)
    }

    fn dealloc_all(&mut self) {
        self.offset = self.data.as_ptr() as usize;
        debug_assert!(self.is_empty());
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn capacity_left(&self) -> usize {
        self.data.as_ptr() as usize + self.data.len() - self.offset
    }
}

impl Owns for Region<'_> {
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.data.as_ptr() <= memory.ptr.as_ptr()
            && memory.ptr.as_ptr() as usize + memory.size <= self.offset as usize
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
        let mut data = [1; 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert!(region.is_empty());

        region
            .alloc(Layout::new::<[u8; 0]>(), AllocInit::Zeroed)
            .expect("Could not allocated 0 bytes");
        assert!(region.is_empty());

        assert_eq!(data, [1; 32]);
    }

    #[test]
    fn alloc_all() {
        let mut data = [1; 32];
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
        let mut data = [1; 32];
        let mut region = Region::new(&mut data);

        assert_eq!(region.capacity(), 32);
        assert_eq!(region.capacity(), region.capacity_left());

        region
            .alloc(Layout::new::<[u8; 16]>(), AllocInit::Zeroed)
            .expect("Could not allocated 16 bytes");
        assert_eq!(region.capacity_left(), 16);

        assert_eq!(&data[0..16], &[0; 16][..]);
        assert_eq!(&data[16..], &[1; 16][..]);
    }

    #[test]
    fn alloc_full() {
        let mut data = [1; 32];
        let mut region = Region::new(&mut data);

        region
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Zeroed)
            .expect("Could not allocated 32 bytes");
        assert_eq!(region.capacity_left(), 0);

        assert_eq!(data, [0; 32]);
    }

    #[test]
    fn alloc_uninitialzed() {
        let mut data = [1; 32];
        let mut region = Region::new(&mut data);

        region
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
            .expect("Could not allocated 32 bytes");
        assert_eq!(region.capacity_left(), 0);

        assert_eq!(data, [1; 32]);
    }

    #[test]
    fn alloc_fail() {
        let mut data = [1; 32];
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
        let mut data = [1; 32];
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
        let mut data = [1; 32];
        let mut region = Region::new(&mut data);
        let layout = Layout::from_size_align(8, 1).expect("Invalid layout");

        let memory = region
            .alloc(layout, AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        assert_eq!(memory.size, 8);
        assert_eq!(region.capacity_left(), 24);

        let memory = unsafe {
            region
                .grow(
                    memory.ptr,
                    layout,
                    16,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 16 bytes")
        };
        assert_eq!(memory.size, 16);
        assert_eq!(region.capacity_left(), 16);

        let memory = unsafe {
            region
                .shrink(
                    memory.ptr,
                    Layout::from_size_align(16, 1).expect("Invalid layout"),
                    8,
                    ReallocPlacement::InPlace,
                )
                .expect("Could not shrink to 8 bytes")
        };
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
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect_err("Could grow 16 bytes in place");
            region
                .grow(
                    memory.ptr,
                    layout,
                    8,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow by 0 bytes");
            region
                .shrink(memory.ptr, layout, 8, ReallocPlacement::InPlace)
                .expect("Could not grow by 0 bytes");
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
        let memory = region.alloc(Layout::new::<[u8; 16]>(), AllocInit::Zeroed).expect("Could not allocate 16 bytes");
        region.alloc(Layout::new::<[u8; 8]>(), AllocInit::Uninitialized).expect("Could not allocate 16 bytes");

        unsafe {
            region.shrink(memory.ptr, Layout::new::<[u8; 16]>(), 8, ReallocPlacement::MayMove).expect("Could not shrink to 8 bytes");
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

        let mut data = [1; 32];
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
