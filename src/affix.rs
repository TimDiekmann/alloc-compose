use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};

/// An allocator that requests some extra memory from the parent allocator for storing
/// a prefix and/or a suffix.
///
/// The alignment of the memory block is the maximum of the alignment of `Prefix` and the requested
/// alignment. This may introduce an unused padding between `Prefix` and the returned memory.
///
/// # Examples
///
/// `Prefix` is `12` bytes in size and has an alignment requirement of `4` bytes. `Suffix` is `16`
/// bytes in size, the requested layout requires `28` bytes, both with an alignment of `8` bytes.
/// The parent allocator returns memory blocks of `128` bytes to demonstrate the behavior on
/// overallocating.
/// ```
/// #![feature(allocator_api)]
///
/// use alloc_compose::{Affix, ChunkAlloc};
/// use std::alloc::{Layout, System};
///
/// type Prefix = [u32; 3];
/// # assert_eq!(core::mem::size_of::<Prefix>(), 12);
/// # assert_eq!(core::mem::align_of::<Prefix>(), 4);
/// type Suffix = [u64; 2];
/// # assert_eq!(core::mem::size_of::<Suffix>(), 16);
/// # assert_eq!(core::mem::align_of::<Suffix>(), 8);
/// type Alloc = Affix<ChunkAlloc<System, 128>, Prefix, Suffix>;
///
/// let layout = Layout::from_size_align(28, 8)?;
/// # Ok::<(), core::alloc::LayoutErr>(())
/// ```
///
/// The memory layout differs depending on `Prefix` and `Suffix`:
///
/// ```
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Affix, ChunkAlloc};
/// # use std::alloc::{Layout, System};
/// use core::alloc::{AllocRef, AllocInit};
/// # type Prefix = [u32; 3];
/// # type Suffix = [u64; 2];
/// # type Alloc = Affix<ChunkAlloc<System, 128>, Prefix, Suffix>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0          12  16                          44  48              64       128
/// // ╞═ Prefix ══╡   ╞════ requested memory ═════╡   ╞═══ Suffix ════╡        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳╅┬┬┬╆┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╅┬┬╌╌╌╌┬┬┤
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┹┴┴┴╄┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┹┴┴╌╌╌╌┴┴┘
/// // │               ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘   │
/// // │               ├┄┄┄┄┄┄┄┄┄ memory.size ┄┄┄┄┄┄┄┄┄┤
/// // └→ prefix()     └→ memory.ptr                   └→ suffix()
/// let memory = my_alloc.alloc(layout, AllocInit::Uninitialized)?;
///
/// assert_eq!(memory.size, 32);
/// unsafe {
///     assert_eq!(Alloc::prefix(memory.ptr, layout).cast().as_ptr(), memory.ptr.as_ptr().sub(16));
///     assert_eq!(Alloc::suffix(memory.ptr, layout).cast().as_ptr(), memory.ptr.as_ptr().add(32));
/// }
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// The memory between `Prefix` and the requested memory is unused. If there is a padding between
/// the requested memory and the suffix, this can be used as extra memory for the allocation. The
/// memory after `Suffix` is also unused as `Suffix` is typed. This results in `68` bytes unused
/// memory.
///
/// If `Suffix` is a zero-sized type, the space after the requested memory block can be used:
///
/// ```
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Affix, ChunkAlloc};
/// # use std::alloc::{Layout, System, AllocRef, AllocInit};
/// use core::ptr::NonNull;
/// # type Prefix = [u32; 3];
///
/// // For convenience, the suffix can be ommitted
/// type Alloc = Affix<ChunkAlloc<System, 128>, Prefix>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0          12  16                          44  48              64       128
/// // ╞═ Prefix ══╡   ╞════ requested memory ═════╡   │               │        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳╅┬┬┬╆┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳╍╍╍╍┳┳┪
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┹┴┴┴╄┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╍╍╍╍┻┻┩
/// // │               ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘                            │
/// // │               ├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ memory.size ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┘
/// // └→ prefix()     └→ memory.ptr
/// let memory = my_alloc.alloc(layout, AllocInit::Uninitialized)?;
///
/// assert_eq!(memory.size, 112);
/// unsafe {
///     assert_eq!(Alloc::prefix(memory.ptr, layout).cast().as_ptr(), memory.ptr.as_ptr().sub(16));
///     assert_eq!(Alloc::suffix(memory.ptr, layout), NonNull::dangling());
/// }
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// This results in only `4` bytes unused memory.
///
/// If `Prefix` is a zero-sized type, this results in a waste of memory:
///
/// ```
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Affix, ChunkAlloc};
/// # use std::alloc::{Layout, System, AllocRef, AllocInit};
/// # use core::ptr::NonNull;
/// # type Suffix = [u64; 2];
/// type Alloc = Affix<ChunkAlloc<System, 128>, (), Suffix>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0                          28  32              48              64       128
/// // ╞════ requested memory ═════╡   ╞═══ Suffix ════╡               │        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╅┬┬┬┬┬┬┬┬┬┬┬┬┬┬┬┼┬┬╌╌╌╌┬┬┤
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┹┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴╌╌╌╌┴┴┘
/// // ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘   │
/// // ├┄┄┄┄┄┄┄┄┄ memory.size ┄┄┄┄┄┄┄┄┄┤
/// // └→ memory.ptr                   └→ suffix()
/// let memory = my_alloc.alloc(layout, AllocInit::Uninitialized)?;
///
/// assert_eq!(memory.size, 32);
/// unsafe {
///     assert_eq!(Alloc::prefix(memory.ptr, layout), NonNull::dangling());
///     assert_eq!(Alloc::suffix(memory.ptr, layout).cast().as_ptr(), memory.ptr.as_ptr().add(32));
/// }
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// This results in 80 bytes unused memory. As can be seen, if possible a prefix should be
/// preferred to the suffix.
///
/// If both, `Prefix` and `Suffix` are ZSTs, this behaves like the parent allocator:
///
/// ```
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Affix, ChunkAlloc};
/// # use std::alloc::{Layout, System, AllocRef, AllocInit};
/// # use core::ptr::NonNull;
/// # type Suffix = [u64; 2];
/// type Alloc = Affix<ChunkAlloc<System, 128>, (), ()>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0                          28  32              48              64       128
/// // ╞════ requested memory ═════╡   │               │               │        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳╍╍╍╍┳┳┪
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╍╍╍╍┻┻┩
/// // ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘                                            │
/// // ├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ memory.size ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┘
/// // └→ memory.ptr
/// let memory = my_alloc.alloc(layout, AllocInit::Uninitialized)?;
///
/// assert_eq!(memory.size, 128);
/// unsafe {
///     assert_eq!(Alloc::prefix(memory.ptr, layout), NonNull::dangling());
///     assert_eq!(Alloc::suffix(memory.ptr, layout), NonNull::dangling());
/// }
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
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

    /// Returns a pointer to the prefix.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator, and
    /// * `layout` must [*fit*] that block of memory.
    pub unsafe fn prefix(ptr: NonNull<u8>, layout: Layout) -> NonNull<Prefix> {
        if mem::size_of::<Prefix>() == 0 {
            NonNull::dangling()
        } else {
            let prefix = Layout::new::<Prefix>();
            let offset = prefix.size() + prefix.padding_needed_for(layout.align());
            NonNull::new_unchecked(ptr.as_ptr().sub(offset)).cast()
        }
    }

    /// Returns a pointer to the suffix.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator, and
    /// * `layout` must [*fit*] that block of memory.
    pub unsafe fn suffix(ptr: NonNull<u8>, layout: Layout) -> NonNull<Suffix> {
        if mem::size_of::<Suffix>() == 0 {
            NonNull::dangling()
        } else {
            let offset = layout.size() + layout.padding_needed_for(mem::align_of::<Suffix>());
            NonNull::new_unchecked(ptr.as_ptr().add(offset)).cast()
        }
    }

    fn extend_layout(layout: Layout) -> Option<(Layout, usize, usize)> {
        let (layout, content_offset) = if mem::size_of::<Prefix>() == 0 {
            (layout, 0)
        } else {
            Layout::new::<Prefix>().extend(layout).ok()?
        };
        let (layout, suffix_offset) = if mem::size_of::<Suffix>() == 0 {
            (layout, 0)
        } else {
            layout.extend(Layout::new::<Suffix>()).ok()?
        };

        Some((layout, content_offset, suffix_offset))
    }
}

unsafe impl<Alloc, Prefix, Suffix> AllocRef for Affix<Alloc, Prefix, Suffix>
where
    Alloc: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        let (layout, offset_prefix, offset_suffix) = Self::extend_layout(layout).ok_or(AllocErr)?;

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
    ) -> Result<MemoryBlock, AllocErr> {
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
    ) -> Result<MemoryBlock, AllocErr> {
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
    use crate::{
        helper::{AsSlice, Tracker},
        Proxy,
    };
    use core::fmt;
    use std::alloc::System;

    #[allow(clippy::too_many_lines)]
    fn test_alloc<Prefix, Suffix>(
        prefix: Prefix,
        layout: Layout,
        suffix: Suffix,
        offset_prefix: usize,
        offset_suffix: usize,
    ) where
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

            if mem::size_of::<Prefix>() == 0 {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::prefix(memory.ptr, layout),
                    NonNull::dangling()
                );
            } else {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::prefix(memory.ptr, layout)
                        .cast()
                        .as_ptr(),
                    memory.ptr.as_ptr().sub(offset_prefix)
                );
            }
            if mem::size_of::<Suffix>() == 0 {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::suffix(memory.ptr, layout),
                    NonNull::dangling()
                );
            } else {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::suffix(memory.ptr, layout)
                        .cast()
                        .as_ptr(),
                    memory.ptr.as_ptr().add(offset_suffix)
                );
            }

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
            assert_eq!(memory.as_slice(), &vec![0_u8; memory.size][..]);
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
                .expect("Could not grow allocation");
            let new_layout =
                Layout::from_size_align(memory.size * 2, layout.align()).expect("Invalid layout");

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(growed_memory.ptr, new_layout).as_ref(),
                &prefix
            );
            assert_eq!(
                growed_memory.as_slice(),
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
            assert_eq!(memory.as_slice(), &vec![0_u8; memory.size][..]);
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(memory.ptr, layout).as_ref(),
                &suffix
            );

            alloc.dealloc(memory.ptr, layout);
        }
    }

    #[test]
    fn test_alloc_u16_u32_u16() {
        test_alloc::<u16, u16>(0xDEDE, Layout::new::<u32>(), 0xEFEF, 4, 4)
    }

    #[test]
    fn test_alloc_zst_u32_zst() {
        test_alloc::<(), ()>((), Layout::new::<u32>(), (), 0, 0)
    }

    #[test]
    fn test_alloc_zst_u32_u16() {
        test_alloc::<(), u16>((), Layout::new::<u32>(), 0xEFEF, 0, 4)
    }

    #[test]
    fn test_alloc_u16_u64_zst() {
        test_alloc::<u16, ()>(0xDEDE, Layout::new::<u32>(), (), 4, 0)
    }

    #[repr(align(1024))]
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct AlignTo1024 {
        a: u16,
    }

    #[repr(align(64))]
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct AlignTo64;

    #[test]
    fn test_alloc_a1024_u32_zst() {
        test_alloc::<AlignTo1024, ()>(AlignTo1024 { a: 0xDEDE }, Layout::new::<u32>(), (), 1024, 0)
    }

    #[test]
    fn test_alloc_u16_u32_a1024() {
        test_alloc::<u16, AlignTo1024>(
            0xDEDE,
            Layout::new::<u32>(),
            AlignTo1024 { a: 0xDEDE },
            4,
            1024,
        )
    }

    #[test]
    fn test_alloc_a64_u32_zst() {
        test_alloc::<AlignTo64, ()>(AlignTo64, Layout::new::<u32>(), (), 0, 0)
    }

    #[test]
    fn test_alloc_u16_u32_a64() {
        test_alloc::<u16, AlignTo64>(0xDEDE, Layout::new::<u32>(), AlignTo64, 4, 0)
    }
}
