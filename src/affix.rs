use crate::{helper::AllocInit, AllocAll, ReallocInPlace};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
    fmt,
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
/// To get a pointer to the prefix or the suffix, the [`prefix()`] and [`suffix()`] may be called.
///
/// [`prefix()`]: Self::prefix
/// [`suffix()`]: Self::suffix
///
/// # Performance
///
/// Generally it's faster to calculate the pointer to the prefix than the pointer to the suffix, as
/// the extended layout of `Prefix` and the requested memory is needed in order to calculate the
/// `Suffix` pointer. Additionally, in most cases it's recommended to use a prefix over a suffix for
/// a more efficient use of memory. However, small prefixes blunt the alignment so if a large
/// alignment with a small affix is needed, suffixes may be the better option.
///
/// For layouts known at compile time the compiler is able to optimize away almost all calculations.
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
/// use alloc_compose::{Affix, Chunk};
/// use std::alloc::{Layout, System};
///
/// type Prefix = [u32; 3];
/// # assert_eq!(core::mem::size_of::<Prefix>(), 12);
/// # assert_eq!(core::mem::align_of::<Prefix>(), 4);
/// type Suffix = [u64; 2];
/// # assert_eq!(core::mem::size_of::<Suffix>(), 16);
/// # assert_eq!(core::mem::align_of::<Suffix>(), 8);
/// type Alloc = Affix<Chunk<System, 128>, Prefix, Suffix>;
///
/// let layout = Layout::from_size_align(28, 8)?;
/// # Ok::<(), core::alloc::LayoutErr>(())
/// ```
///
/// The memory layout differs depending on `Prefix` and `Suffix`:
///
/// ```
/// #![feature(slice_ptr_get, slice_ptr_len)]
/// # #![feature(allocator_api)]
/// # use alloc_compose::{Affix, Chunk};
/// # use std::alloc::{Layout, System};
///
/// use core::alloc::AllocRef;
/// # type Prefix = [u32; 3];
/// # type Suffix = [u64; 2];
/// # type Alloc = Affix<Chunk<System, 128>, Prefix, Suffix>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0          12  16                          44  48              64       128
/// // ╞═ Prefix ══╡   ╞════ requested memory ═════╡   ╞═══ Suffix ════╡        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳╅┬┬┬╆┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╅┬┬╌╌╌╌┬┬┤
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┹┴┴┴╄┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┹┴┴╌╌╌╌┴┴┘
/// // │               ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘   │
/// // │               ├┄┄┄┄┄┄┄┄ memory.len() ┄┄┄┄┄┄┄┄┄┤
/// // └→ prefix()     └→ memory                       └→ suffix()
/// let memory = my_alloc.alloc(layout)?;
///
/// assert_eq!(memory.len(), 32);
/// unsafe {
///     assert_eq!(
///         Alloc::prefix(memory.as_non_null_ptr(), layout).cast().as_ptr(),
///         memory.as_mut_ptr().sub(16)
///     );
///     assert_eq!(
///         Alloc::suffix(memory.as_non_null_ptr(), layout).cast().as_ptr(),
///         memory.as_mut_ptr().add(32)
///     );
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
/// # #![feature(allocator_api, slice_ptr_get, slice_ptr_len)]
/// # use alloc_compose::{Affix, Chunk};
/// # use std::alloc::{Layout, System, AllocRef};
/// use core::ptr::NonNull;
/// # type Prefix = [u32; 3];
///
/// // For convenience, the suffix can be ommitted
/// type Alloc = Affix<Chunk<System, 128>, Prefix>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0          12  16                          44  48              64       128
/// // ╞═ Prefix ══╡   ╞════ requested memory ═════╡   │               │        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳╅┬┬┬╆┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳╍╍╍╍┳┳┪
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┹┴┴┴╄┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╍╍╍╍┻┻┩
/// // │               ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘                            │
/// // │               ├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ memory.len() ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┘
/// // └→ prefix()     └→ memory
/// let memory = my_alloc.alloc(layout)?;
///
/// assert_eq!(memory.len(), 112);
/// unsafe {
///     assert_eq!(
///         Alloc::prefix(memory.as_non_null_ptr(), layout).cast().as_ptr(),
///         memory.as_mut_ptr().sub(16)
///     );
///     assert_eq!(Alloc::suffix(memory.as_non_null_ptr(), layout), NonNull::dangling());
/// }
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// This results in only `4` bytes unused memory.
///
/// If `Prefix` is a zero-sized type, this results in a waste of memory:
///
/// ```
/// # #![feature(allocator_api, slice_ptr_get, slice_ptr_len)]
/// # use alloc_compose::{Affix, Chunk};
/// # use std::alloc::{Layout, System, AllocRef};
/// # use core::ptr::NonNull;
/// # type Suffix = [u64; 2];
/// type Alloc = Affix<Chunk<System, 128>, (), Suffix>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0                          28  32              48              64       128
/// // ╞════ requested memory ═════╡   ╞═══ Suffix ════╡               │        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╅┬┬┬┬┬┬┬┬┬┬┬┬┬┬┬┼┬┬╌╌╌╌┬┬┤
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┹┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴┴╌╌╌╌┴┴┘
/// // ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘   │
/// // ├┄┄┄┄┄┄┄┄ memory.len() ┄┄┄┄┄┄┄┄┄┤
/// // └→ memory                       └→ suffix()
/// let memory = my_alloc.alloc(layout)?;
///
/// assert_eq!(memory.len(), 32);
/// unsafe {
///     assert_eq!(Alloc::prefix(memory.as_non_null_ptr(), layout), NonNull::dangling());
///     assert_eq!(
///         Alloc::suffix(memory.as_non_null_ptr(), layout).cast().as_ptr(),
///         memory.as_mut_ptr().add(32)
///     );
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
/// # #![feature(allocator_api, slice_ptr_get, slice_ptr_len)]
/// # use alloc_compose::{Affix, Chunk};
/// # use std::alloc::{Layout, System, AllocRef};
/// # use core::ptr::NonNull;
/// # type Suffix = [u64; 2];
/// type Alloc = Affix<Chunk<System, 128>, (), ()>;
/// # let layout = Layout::from_size_align(28, 8).unwrap();
///
/// let mut my_alloc = Alloc::default();
///
/// // 0                          28  32              48              64       128
/// // ╞════ requested memory ═════╡   │               │               │        │
/// // ┢┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳┳┳┳┳┳┳┳┳┳┳┳┳┳╈┳┳╍╍╍╍┳┳┪
/// // ┡┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╇┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻┻╍╍╍╍┻┻┩
/// // ├┄┄┄┄┄┄ layout.size() ┄┄┄┄┄┄┘                                            │
/// // ├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ memory.len() ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┘
/// // └→ memory
/// let memory = my_alloc.alloc(layout)?;
///
/// assert_eq!(memory.len(), 128);
/// unsafe {
///     assert_eq!(Alloc::prefix(memory.as_non_null_ptr(), layout), NonNull::dangling());
///     assert_eq!(Alloc::suffix(memory.as_non_null_ptr(), layout), NonNull::dangling());
/// }
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
pub struct Affix<Alloc, Prefix = (), Suffix = ()> {
    /// The parent allocator to be used as backend
    pub parent: Alloc,
    _prefix: PhantomData<Prefix>,
    _suffix: PhantomData<Suffix>,
}

impl<Alloc: fmt::Debug, Prefix, Suffix> fmt::Debug for Affix<Alloc, Prefix, Suffix> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Affix")
            .field("parent", &self.parent)
            .finish()
    }
}

impl<Alloc: Default, Prefix, Suffix> Default for Affix<Alloc, Prefix, Suffix> {
    fn default() -> Self {
        Self::new(Alloc::default())
    }
}

impl<Alloc: Clone, Prefix, Suffix> Clone for Affix<Alloc, Prefix, Suffix> {
    fn clone(&self) -> Self {
        Self::new(self.parent.clone())
    }
}

impl<Alloc: Copy, Prefix, Suffix> Copy for Affix<Alloc, Prefix, Suffix> {}

impl<Alloc: PartialEq, Prefix, Suffix> PartialEq for Affix<Alloc, Prefix, Suffix> {
    fn eq(&self, other: &Self) -> bool {
        self.parent.eq(&other.parent)
    }
}

impl<Alloc: Eq, Prefix, Suffix> Eq for Affix<Alloc, Prefix, Suffix> {}

unsafe impl<Alloc: Send, Prefix, Suffix> Send for Affix<Alloc, Prefix, Suffix> {}
unsafe impl<Alloc: Sync, Prefix, Suffix> Sync for Affix<Alloc, Prefix, Suffix> {}
impl<Alloc: Unpin, Prefix, Suffix> Unpin for Affix<Alloc, Prefix, Suffix> {}

impl<Alloc, Prefix, Suffix> Affix<Alloc, Prefix, Suffix> {
    pub const fn new(parent: Alloc) -> Self {
        Self {
            parent,
            _prefix: PhantomData,
            _suffix: PhantomData,
        }
    }

    fn allocation_layout(layout: Layout) -> Option<(Layout, usize, usize)> {
        let (layout, prefix_offset) = Layout::new::<Prefix>().extend(layout).ok()?;
        let (layout, suffix_offset) = layout.extend(Layout::new::<Suffix>()).ok()?;
        Some((layout, prefix_offset, suffix_offset))
    }

    /// Returns a pointer to the prefix.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory *[currently allocated]* via this allocator, and
    /// * `layout` must *[fit]* that block of memory.
    ///
    /// [currently allocated]: https://doc.rust-lang.org/nightly/core/alloc/trait.AllocRef.html#currently-allocated-memory
    /// [fit]: https://doc.rust-lang.org/nightly/core/alloc/trait.AllocRef.html#memory-fitting
    pub unsafe fn prefix(ptr: NonNull<u8>, layout: Layout) -> NonNull<Prefix> {
        if mem::size_of::<Prefix>() == 0 {
            NonNull::dangling()
        } else {
            let (_, prefix, _) = Self::allocation_layout(layout).unwrap();
            NonNull::new_unchecked(ptr.as_ptr().sub(prefix)).cast()
        }
    }

    /// Returns a pointer to the suffix.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory *[currently allocated]* via this allocator, and
    /// * `layout` must *[fit]* that block of memory.
    ///
    /// [currently allocated]: https://doc.rust-lang.org/nightly/core/alloc/trait.AllocRef.html#currently-allocated-memory
    /// [fit]: https://doc.rust-lang.org/nightly/core/alloc/trait.AllocRef.html#memory-fitting
    pub unsafe fn suffix(ptr: NonNull<u8>, layout: Layout) -> NonNull<Suffix> {
        if mem::size_of::<Suffix>() == 0 {
            NonNull::dangling()
        } else {
            let (_, prefix, suffix) = Self::allocation_layout(layout).unwrap();
            NonNull::new_unchecked(ptr.as_ptr().add(suffix - prefix)).cast()
        }
    }

    fn create_ptr(ptr: NonNull<[u8]>, offset_prefix: usize, offset_suffix: usize) -> NonNull<[u8]> {
        let len = if mem::size_of::<Suffix>() == 0 {
            ptr.len() - offset_prefix
        } else {
            offset_suffix - offset_prefix
        };
        let ptr = unsafe { NonNull::new_unchecked(ptr.as_mut_ptr().add(offset_prefix)) };

        NonNull::slice_from_raw_parts(ptr, len)
    }

    #[inline]
    fn alloc_impl(
        layout: Layout,
        alloc: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocErr>,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let (layout, offset_prefix, offset_suffix) =
            Self::allocation_layout(layout).ok_or(AllocErr)?;

        Ok(Self::create_ptr(
            alloc(layout)?,
            offset_prefix,
            offset_suffix,
        ))
    }

    #[inline]
    unsafe fn grow_impl(
        old_ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
        init: AllocInit,
        grow: impl FnOnce(NonNull<u8>, Layout, usize) -> Result<NonNull<[u8]>, AllocErr>,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let (old_alloc_layout, old_offset_prefix, old_offset_suffix) =
            Self::allocation_layout(old_layout).ok_or(AllocErr)?;
        let old_base_ptr = NonNull::new_unchecked(old_ptr.as_ptr().sub(old_offset_prefix));

        let suffix = Self::suffix(old_ptr, old_layout)
            .cast::<MaybeUninit<Suffix>>()
            .as_ptr()
            .read();

        let new_layout =
            Layout::from_size_align(new_size, old_layout.align()).map_err(|_| AllocErr)?;
        let (new_alloc_layout, new_offset_prefix, new_offset_suffix) =
            Self::allocation_layout(new_layout).ok_or(AllocErr)?;

        let new_base_ptr = grow(old_base_ptr, old_alloc_layout, new_alloc_layout.size())?;

        if init == AllocInit::Zeroed {
            ptr::write_bytes(
                new_base_ptr
                    .as_non_null_ptr()
                    .as_ptr()
                    .add(old_offset_suffix),
                0,
                mem::size_of::<Suffix>(),
            );
        }

        let new_ptr = Self::create_ptr(new_base_ptr, new_offset_prefix, new_offset_suffix);

        Self::suffix(new_ptr.as_non_null_ptr(), new_layout)
            .cast::<MaybeUninit<Suffix>>()
            .as_ptr()
            .write(suffix);

        Ok(new_ptr)
    }

    #[inline]
    unsafe fn shrink_impl(
        old_ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
        shrink: impl FnOnce(NonNull<u8>, Layout, usize) -> Result<NonNull<[u8]>, AllocErr>,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let (old_alloc_layout, old_offset_prefix, _) =
            Self::allocation_layout(old_layout).ok_or(AllocErr)?;
        let old_base_ptr = NonNull::new_unchecked(old_ptr.as_ptr().sub(old_offset_prefix));

        let suffix = Self::suffix(old_ptr, old_layout)
            .cast::<MaybeUninit<Suffix>>()
            .as_ptr()
            .read();

        let new_layout =
            Layout::from_size_align(new_size, old_layout.align()).map_err(|_| AllocErr)?;
        let (new_alloc_layout, new_offset_prefix, new_offset_suffix) =
            Self::allocation_layout(new_layout).ok_or(AllocErr)?;

        let new_base_ptr = shrink(old_base_ptr, old_alloc_layout, new_alloc_layout.size())?;

        let new_ptr = Self::create_ptr(new_base_ptr, new_offset_prefix, new_offset_suffix);

        Self::suffix(new_ptr.as_non_null_ptr(), new_layout)
            .cast::<MaybeUninit<Suffix>>()
            .as_ptr()
            .write(suffix);

        Ok(new_ptr)
    }
}

unsafe impl<Alloc, Prefix, Suffix> AllocRef for Affix<Alloc, Prefix, Suffix>
where
    Alloc: AllocRef,
{
    impl_alloc_ref!(parent);

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let (layout, prefix_offset, _) = Self::allocation_layout(layout).unwrap();
        let base_ptr = ptr.as_ptr().sub(prefix_offset);
        self.parent
            .dealloc(NonNull::new_unchecked(base_ptr), layout)
    }
}

unsafe impl<Alloc, Prefix, Suffix> AllocAll for Affix<Alloc, Prefix, Suffix>
where
    Alloc: AllocAll,
{
    impl_alloc_all!(parent);
}

unsafe impl<Alloc, Prefix, Suffix> ReallocInPlace for Affix<Alloc, Prefix, Suffix>
where
    Alloc: ReallocInPlace,
{
    impl_realloc_in_place!(parent);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports)]
    use super::*;
    use crate::helper::tracker;
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
            let mut alloc = tracker(Affix::<_, Prefix, Suffix>::new(tracker(System)));
            let memory = alloc
                .alloc_zeroed(layout)
                .unwrap_or_else(|_| panic!("Could not allocate {} bytes", layout.size()));

            if mem::size_of::<Prefix>() == 0 {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::prefix(memory.as_non_null_ptr(), layout),
                    NonNull::dangling()
                );
            } else {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::prefix(memory.as_non_null_ptr(), layout)
                        .cast()
                        .as_ptr(),
                    memory.as_mut_ptr().sub(offset_prefix)
                );
            }
            if mem::size_of::<Suffix>() == 0 {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::suffix(memory.as_non_null_ptr(), layout),
                    NonNull::dangling()
                );
            } else {
                assert_eq!(
                    Affix::<System, Prefix, Suffix>::suffix(memory.as_non_null_ptr(), layout)
                        .cast()
                        .as_ptr(),
                    memory.as_mut_ptr().add(offset_suffix)
                );
            }

            Affix::<System, Prefix, Suffix>::prefix(memory.as_non_null_ptr(), layout)
                .as_ptr()
                .write(prefix);
            Affix::<System, Prefix, Suffix>::suffix(memory.as_non_null_ptr(), layout)
                .as_ptr()
                .write(suffix);

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(memory.as_non_null_ptr(), layout).as_ref(),
                &prefix
            );
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(memory.as_non_null_ptr(), layout).as_ref(),
                &suffix
            );

            let old_size = memory.len();
            let memory = alloc
                .grow_zeroed(memory.as_non_null_ptr(), layout, memory.len() * 2)
                .expect("Could not grow allocation");
            let layout =
                Layout::from_size_align(memory.len(), layout.align()).expect("Invalid layout");

            for i in old_size..memory.len() {
                assert_eq!(*memory.get_unchecked_mut(i).as_ref(), 0);
            }

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(memory.as_non_null_ptr(), layout).as_ref(),
                &prefix
            );
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(memory.as_non_null_ptr(), layout).as_ref(),
                &suffix
            );

            let memory = alloc
                .shrink(memory.as_non_null_ptr(), layout, layout.size())
                .expect("Could not shrink allocation");
            let layout =
                Layout::from_size_align(memory.len(), layout.align()).expect("Invalid layout");

            assert_eq!(
                Affix::<System, Prefix, Suffix>::prefix(memory.as_non_null_ptr(), layout).as_ref(),
                &prefix
            );
            assert_eq!(
                Affix::<System, Prefix, Suffix>::suffix(memory.as_non_null_ptr(), layout).as_ref(),
                &suffix
            );

            alloc.dealloc(memory.as_non_null_ptr(), layout);
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
            AlignTo1024 { a: 0xEFEF },
            4,
            1020,
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
