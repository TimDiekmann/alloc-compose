use crate::Result;
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, LayoutErr, MemoryBlock, ReallocPlacement},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Affix<Alloc, Prefix = (), Suffix = ()> {
    pub alloc: Alloc,
    _prefix: PhantomData<Prefix>,
    _suffix: PhantomData<Suffix>,
}

impl<Alloc, Prefix, Suffix> Default for Affix<Alloc, Prefix, Suffix>
where
    Alloc: Default,
{
    fn default() -> Self {
        Self::new(Alloc::default())
    }
}
impl<Alloc, Prefix, Suffix> Affix<Alloc, Prefix, Suffix> {
    pub const fn new(alloc: Alloc) -> Self {
        Self {
            alloc,
            _prefix: PhantomData,
            _suffix: PhantomData,
        }
    }

    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator, and
    /// * `layout` must [*fit*] that block of memory.
    pub unsafe fn prefix(ptr: NonNull<u8>, layout: Layout) -> NonNull<Prefix> {
        let prefix = Layout::new::<Prefix>();
        let offset = prefix.size() + prefix.padding_needed_for(layout.align());
        NonNull::new_unchecked(ptr.as_ptr().sub(offset)).cast()
    }

    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator, and
    /// * `layout` must [*fit*] that block of memory.
    pub unsafe fn suffix(ptr: NonNull<u8>, layout: Layout) -> NonNull<Suffix> {
        let offset = layout.size() + layout.padding_needed_for(mem::align_of::<Suffix>());
        NonNull::new_unchecked(ptr.as_ptr().add(offset)).cast()
    }

    fn extend_layout(layout: Layout) -> Result<(Layout, usize, usize), LayoutErr> {
        let prefix_layout = Layout::new::<Prefix>();
        let suffix_layout = Layout::new::<Suffix>();

        let (layout, prefix_offset) = prefix_layout.extend(layout)?;
        let (layout, suffix_offset) = layout.extend(suffix_layout)?;

        Ok((layout, prefix_offset, suffix_offset))
    }
}

unsafe impl<Alloc, Prefix, Suffix> AllocRef for Affix<Alloc, Prefix, Suffix>
where
    Alloc: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result {
        let (layout, offset_prefix, offset_suffix) =
            Self::extend_layout(layout).map_err(|_| AllocErr)?;

        let memory = self.alloc.alloc(layout, init)?;

        Ok(MemoryBlock {
            ptr: unsafe { NonNull::new_unchecked(memory.ptr.as_ptr().add(offset_prefix)) },
            size: if mem::size_of::<Suffix>() == 0 {
                memory.size - offset_prefix
            } else {
                offset_suffix - offset_prefix
            },
        })
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let (layout, prefix_offset, _) = Self::extend_layout(layout).expect("Invalid layout");
        let base_ptr = ptr.as_ptr().sub(prefix_offset);
        self.alloc.dealloc(NonNull::new_unchecked(base_ptr), layout)
    }

    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result {
        let (old_alloc_layout, old_offset_prefix, old_offset_suffix) =
            Self::extend_layout(old_layout).unwrap();
        let ptr = ptr.as_ptr().sub(old_offset_prefix);

        let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());
        let (new_alloc_layout, new_offset_prefix, new_offset_suffix) =
            Self::extend_layout(new_layout).unwrap();

        let suffix: MaybeUninit<Suffix> = ptr::read(ptr.add(old_offset_suffix) as *mut _);
        let memory = self.alloc.grow(
            NonNull::new_unchecked(ptr),
            old_alloc_layout,
            new_alloc_layout.size(),
            placement,
            init,
        )?;

        if init == AllocInit::Zeroed {
            ptr::write_bytes(
                memory.ptr.as_ptr().add(old_offset_suffix),
                0,
                mem::size_of::<Suffix>(),
            );
        }
        ptr::write(memory.ptr.as_ptr().add(new_offset_suffix) as *mut _, suffix);

        Ok(MemoryBlock {
            ptr: NonNull::new_unchecked(memory.ptr.as_ptr().add(new_offset_prefix)),
            size: if mem::size_of::<Suffix>() == 0 {
                memory.size - new_offset_prefix
            } else {
                new_offset_suffix - new_offset_prefix
            },
        })
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result {
        let (old_alloc_layout, old_offset_prefix, old_offset_suffix) =
            Self::extend_layout(old_layout).unwrap();
        let ptr = ptr.as_ptr().sub(old_offset_prefix);

        let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());
        let (new_alloc_layout, new_offset_prefix, new_offset_suffix) =
            Self::extend_layout(new_layout).unwrap();

        let suffix: MaybeUninit<Suffix> = ptr::read(ptr.add(old_offset_suffix) as *mut _);
        let memory = self.alloc.shrink(
            NonNull::new_unchecked(ptr),
            old_alloc_layout,
            new_alloc_layout.size(),
            placement,
        )?;

        ptr::write(memory.ptr.as_ptr().add(new_offset_suffix) as *mut _, suffix);

        Ok(MemoryBlock {
            ptr: NonNull::new_unchecked(memory.ptr.as_ptr().add(new_offset_prefix)),
            size: if mem::size_of::<Suffix>() == 0 {
                memory.size - new_offset_prefix
            } else {
                new_offset_suffix - new_offset_prefix
            },
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]
    use super::*;
    use crate::{helper::Tracker, Proxy};
    use core::{fmt, slice};
    use std::alloc::System;

    #[allow(clippy::too_many_lines)]
    fn test_alloc<Prefix, Suffix>(prefix: Prefix, layout: Layout, suffix: Suffix)
    where
        Prefix: fmt::Debug + Copy + PartialEq,
        Suffix: fmt::Debug + Copy + PartialEq,
    {
        unsafe {
            let mut alloc = Proxy {
                alloc: Affix::<System, Prefix, Suffix>::default(),
                callbacks: Tracker::default(),
            };
            let memory = alloc
                .alloc(layout, AllocInit::Zeroed)
                .unwrap_or_else(|_| panic!("Could not allocate {} bytes", layout.size()));

            Affix::<System, Prefix, Suffix>::prefix(memory.ptr, layout)
                .as_ptr()
                .write(prefix);
            Affix::<System, Prefix, Suffix>::suffix(memory.ptr, layout)
                .as_ptr()
                .write(suffix);

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(memory.ptr, layout).as_ref(),
                &prefix
            );
            assert_eq!(
                slice::from_raw_parts(memory.ptr.as_ptr(), memory.size),
                &vec![0_u8; memory.size][..]
            );
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(memory.ptr, layout).as_ref(),
                &suffix
            );

            let growed_memory = alloc
                .grow(
                    memory.ptr,
                    layout,
                    memory.size * 2,
                    ReallocPlacement::MayMove,
                    AllocInit::Zeroed,
                )
                .unwrap_or_else(|_| {
                    panic!("Could not grow allocation to {} bytes", memory.size * 2)
                });
            let new_layout =
                Layout::from_size_align(memory.size * 2, layout.align()).expect("Invalid layout");

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(growed_memory.ptr, new_layout).as_ref(),
                &prefix
            );
            assert_eq!(
                slice::from_raw_parts(growed_memory.ptr.as_ptr(), growed_memory.size),
                &vec![0_u8; growed_memory.size][..]
            );
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(growed_memory.ptr, new_layout).as_ref(),
                &suffix
            );

            let memory = alloc
                .shrink(
                    growed_memory.ptr,
                    new_layout,
                    layout.size(),
                    ReallocPlacement::MayMove,
                )
                .expect("Could not shrink allocation");

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(memory.ptr, layout).as_ref(),
                &prefix
            );
            assert_eq!(
                slice::from_raw_parts(memory.ptr.as_ptr(), memory.size),
                &vec![0_u8; memory.size][..]
            );
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(memory.ptr, layout).as_ref(),
                &suffix
            );

            alloc.dealloc(memory.ptr, layout);
        }
    }

    #[test]
    fn test_alloc_u32_u64_u32() {
        test_alloc::<u32, u32>(0xDEDE_DEDE, Layout::new::<u64>(), 0xEFEF_EFEF)
    }

    #[test]
    fn test_alloc_zst_u64_zst() {
        test_alloc::<(), ()>((), Layout::new::<u64>(), ())
    }

    #[test]
    fn test_alloc_zst_u64_u32() {
        test_alloc::<(), u32>((), Layout::new::<u64>(), 0xEFEF_EFEF)
    }

    #[test]
    fn test_alloc_u32_u64_zst() {
        test_alloc::<u32, ()>(0xDEDE_DEDE, Layout::new::<u64>(), ())
    }

    #[repr(align(1024))]
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct AlignTo1024 {
        a: u32,
    }

    #[repr(align(16))]
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct AlignTo16;

    #[test]
    fn test_alloc_a1024_u64_zst() {
        test_alloc::<AlignTo1024, ()>(AlignTo1024 { a: 0xDEDE_DEDE }, Layout::new::<u64>(), ())
    }

    #[test]
    fn test_alloc_u32_u64_a1024() {
        test_alloc::<u32, AlignTo1024>(0xDEDE_DEDE, Layout::new::<u64>(), AlignTo1024 {
            a: 0xDEDE_DEDE,
        })
    }

    #[test]
    fn test_alloc_a16_u64_zst() {
        test_alloc::<AlignTo16, ()>(AlignTo16, Layout::new::<u64>(), ())
    }

    #[test]
    fn test_alloc_u32_u64_a16() {
        test_alloc::<u32, AlignTo16>(0xDEDE_DEDE, Layout::new::<u64>(), AlignTo16)
    }
}
