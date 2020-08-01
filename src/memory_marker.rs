use crate::Owns;
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// Marks newly allocated and deallocated memory with a byte pattern.
///
/// When allocating unintitialized memory, the block is set to `0xCD`. Before deallocating,
/// the memory is set `0xDD`.
/// Those values are choosed according to [Magic Debug Values] to match the Visual
/// Studio Debug Heap implementation.
///
/// Once, `const_generics` allows default implementations, the values may be alterd with a parameter.
///
/// [Magic Debug Values]: https://en.wikipedia.org/wiki/Magic_number_%28programming%29#Magic_debug_values
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct MemoryMarker<A>(pub A);

unsafe impl<A: AllocRef> AllocRef for MemoryMarker<A> {
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        let memory = self.0.alloc(layout, init)?;
        if init == AllocInit::Uninitialized {
            unsafe { memory.ptr.as_ptr().write_bytes(0xCD, memory.size) };
        }
        Ok(memory)
    }
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        ptr.as_ptr().write_bytes(0xDD, layout.size());
        self.0.dealloc(ptr, layout)
    }
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        let memory = self.0.grow(ptr, layout, new_size, placement, init)?;
        if init == AllocInit::Uninitialized {
            memory
                .ptr
                .as_ptr()
                .add(layout.size())
                .write_bytes(0xCD, memory.size - layout.size());
        }
        Ok(memory)
    }
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        ptr.as_ptr()
            .add(new_size)
            .write_bytes(0xDD, layout.size() - new_size);
        self.0.shrink(ptr, layout, new_size, placement)
    }
}

impl<A: Owns> Owns for MemoryMarker<A> {
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.0.owns(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryMarker;
    use crate::{
        helper::{self, AsSlice},
        Region,
    };
    use std::alloc::{AllocInit, AllocRef, Layout, ReallocPlacement, System};

    #[test]
    fn alloc() {
        let mut alloc = helper::tracker(MemoryMarker(System));
        let memory = alloc
            .alloc(Layout::new::<u64>(), AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        unsafe {
            assert_eq!(memory.as_slice(), &[0xCD; 8][..]);
            alloc.dealloc(memory.ptr, Layout::new::<u64>());
        }

        let memory = alloc
            .alloc(Layout::new::<u64>(), AllocInit::Zeroed)
            .expect("Could not allocate 8 bytes");
        unsafe {
            assert_eq!(memory.as_slice(), &[0; 8][..]);
            alloc.dealloc(memory.ptr, Layout::new::<u64>());
        }
    }

    #[test]
    fn dealloc() {
        let mut data = [0; 8];
        let mut alloc = helper::tracker(MemoryMarker(Region::new(&mut data)));
        let memory = alloc
            .alloc(Layout::new::<[u8; 8]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 8 bytes");
        unsafe {
            assert_eq!(memory.as_slice(), &[0xCD; 8][..]);
            alloc.dealloc(memory.ptr, Layout::new::<[u8; 8]>());
        }
        drop(alloc);
        assert_eq!(data, [0xDD; 8]);
    }

    #[test]
    fn grow() {
        let mut alloc = helper::tracker(MemoryMarker(System));
        let memory = alloc
            .alloc(Layout::new::<[u64; 4]>(), AllocInit::Zeroed)
            .expect("Could not allocate 32 bytes");
        unsafe {
            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u64; 4]>(),
                    64,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 64 bytes");
            assert_eq!(&memory.as_slice()[..32], &[0; 32][..]);
            assert_eq!(&memory.as_slice()[32..], &[0xCD; 32][..]);
            alloc.dealloc(memory.ptr, Layout::new::<[u64; 8]>());
        }
    }

    #[test]
    fn shrink() {
        let mut data = [0; 8];
        let mut alloc = MemoryMarker(Region::new(&mut data));
        let memory = alloc
            .alloc(Layout::new::<[u8; 8]>(), AllocInit::Zeroed)
            .expect("Could not allocate 8 bytes");
        unsafe {
            alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 8]>(),
                    4,
                    ReallocPlacement::MayMove,
                )
                .expect("Could not shrink to 4 bytes");
            assert_eq!(data, [0, 0, 0, 0, 0xDD, 0xDD, 0xDD, 0xDD]);
        }
    }
}
