use crate::{helper::AllocInit, intrinsics, AllocAll, Owns, ReallocInPlace};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
    ptr::NonNull,
};

/// Allocate memory with a multiple size of the provided chunk size.
///
/// # Examples
///
/// ```rust
/// #![feature(allocator_api, slice_ptr_len)]
///
/// use alloc_compose::{Chunk, Region};
/// use std::{
///     alloc::{AllocRef, Layout},
///     mem::MaybeUninit,
/// };
///
/// let mut data = [MaybeUninit::new(0); 64];
/// let mut alloc = Chunk::<_, 64>(Region::new(&mut data));
/// let ptr = alloc.alloc(Layout::new::<[u8; 16]>())?;
/// assert_eq!(ptr.len() % 32, 0);
/// assert!(ptr.len() >= 32);
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// When growing or shrinking the memory, `Chunk` will try to alter
/// the memory in place before delegating to the underlying allocator.
///
/// ```rust
/// #![feature(slice_ptr_get)]
/// # #![feature(allocator_api, slice_ptr_len)]
/// # use alloc_compose::{Chunk, Region};
/// # use std::{alloc::{AllocRef, Layout}, mem::MaybeUninit};
/// # let mut data = [MaybeUninit::new(0); 64];
/// # let mut alloc = Chunk::<_, 64>(Region::new(&mut data));
/// # let ptr = alloc.alloc(Layout::new::<[u8; 16]>())?;
///
/// use alloc_compose::ReallocInPlace;
///
/// let len = unsafe { alloc.grow_in_place(ptr.as_non_null_ptr(), Layout::new::<[u8; 16]>(), 24)? };
/// assert_eq!(len % 32, 0);
/// assert!(len >= 32);
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Chunk<A, const SIZE: usize>(pub A);

mod sealed {
    pub trait SizeIsPowerOfTwo {}
}
use sealed::SizeIsPowerOfTwo;

macro_rules! is_power_of_two {
    ($($N:literal)+) => {
        $(
            impl<A> SizeIsPowerOfTwo for super::Chunk<A, { usize::pow(2, $N) }> {}
        )+
    };
}

is_power_of_two!(1 2 3 4 5 6 7);
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
is_power_of_two!(8 9 10 11 12 13 14 15);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
is_power_of_two!(16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31);
#[cfg(target_pointer_width = "64")]
is_power_of_two!(32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63);

impl<A, const SIZE: usize> Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    fn round_up(size: usize) -> Result<usize, AllocErr> {
        Ok((size.checked_add(SIZE).ok_or(AllocErr)? - 1) & !(SIZE - 1))
    }

    unsafe fn round_up_unchecked(size: usize) -> usize {
        let new_size = (size.wrapping_add(SIZE) - 1) & !(SIZE - 1);
        debug_assert_eq!(new_size, Self::round_up(size).unwrap());
        new_size
    }

    const fn round_down(size: usize) -> usize {
        size & !(SIZE - 1)
    }

    const fn round_down_ptr(ptr: NonNull<[u8]>) -> NonNull<[u8]> {
        NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), Self::round_down(ptr.len()))
    }

    fn layout(layout: Layout, size: usize) -> Result<Layout, AllocErr> {
        // SAFETY: layout already has valid alignment
        unsafe {
            intrinsics::assume(layout.align().is_power_of_two());
        }

        Layout::from_size_align(size, layout.align()).map_err(|_| AllocErr)
    }

    #[inline]
    fn alloc_impl(
        old_layout: Layout,
        alloc: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocErr>,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let new_size = Self::round_up(old_layout.size())?;
        let new_layout = Self::layout(old_layout, new_size)?;

        alloc(new_layout).map(Self::round_down_ptr)
    }

    #[inline]
    unsafe fn grow_impl(
        old_ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        init: AllocInit,
        grow: impl FnOnce(NonNull<u8>, Layout, usize) -> Result<NonNull<[u8]>, AllocErr>,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let current_size = Self::round_up_unchecked(layout.size());
        if new_size <= current_size {
            let ptr = NonNull::slice_from_raw_parts(old_ptr, current_size);
            init.init_offset(ptr, layout.size());
            return Ok(ptr);
        }

        grow(old_ptr, layout, Self::round_up(new_size)?).map(Self::round_down_ptr)
    }

    #[inline]
    unsafe fn shrink_impl(
        old_ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        shrink: impl FnOnce(NonNull<u8>, Layout, usize) -> Result<NonNull<[u8]>, AllocErr>,
    ) -> Result<NonNull<[u8]>, AllocErr> {
        let current_size = Self::round_up_unchecked(layout.size());
        if new_size > current_size - SIZE {
            return Ok(NonNull::slice_from_raw_parts(old_ptr, current_size));
        }

        shrink(old_ptr, layout, Self::round_up_unchecked(new_size)).map(Self::round_down_ptr)
    }
}

unsafe impl<A: AllocRef, const SIZE: usize> AllocRef for Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    impl_alloc_ref!(0);

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        crate::check_dealloc_precondition(ptr, layout);

        self.0.dealloc(
            ptr,
            Layout::from_size_align_unchecked(
                Self::round_up_unchecked(layout.size()),
                layout.align(),
            ),
        )
    }
}

unsafe impl<A: AllocAll, const SIZE: usize> AllocAll for Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    impl_alloc_all!(0);
}

unsafe impl<A: ReallocInPlace, const SIZE: usize> ReallocInPlace for Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    impl_realloc_in_place!(0);
}

impl<A: Owns, const SIZE: usize> Owns for Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    fn owns(&self, memory: NonNull<[u8]>) -> bool {
        self.0.owns(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::Chunk;
    use crate::{helper, AllocAll, ReallocInPlace, Region};
    use std::{
        alloc::{AllocRef, Layout, System},
        mem::MaybeUninit,
    };

    #[test]
    fn alloc() {
        let mut alloc = helper::tracker(Chunk::<_, 64>(System));
        let memory = alloc
            .alloc(Layout::new::<u8>())
            .expect("Could not allocate 64 bytes");
        assert_eq!(memory.len() % 64, 0);
        assert!(memory.len() >= 64);

        unsafe {
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<u8>());
        }
    }

    #[test]
    fn dealloc() {
        let mut alloc = helper::tracker(Chunk::<_, 64>(System));

        unsafe {
            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>())
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 4]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>())
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 32]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>())
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>())
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>());
        }
    }

    #[test]
    fn grow() {
        let mut data = [MaybeUninit::new(0); 256];
        let mut alloc = helper::tracker(Chunk::<_, 64>(Region::new(&mut data)));

        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        assert_eq!(memory.len() % 64, 0);
        assert_eq!(alloc.capacity_left(), 192);

        let _cannot_grow_in_place = alloc
            .alloc(Layout::new::<[u8; 1]>())
            .expect("Could not allocate 64 bytes");
        assert_eq!(alloc.capacity_left(), 128);

        unsafe {
            let len = alloc
                .grow_in_place(memory.as_non_null_ptr(), Layout::new::<[u8; 4]>(), 8)
                .expect("Could not grow to 8 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 64);

            let len = alloc
                .grow_in_place(memory.as_non_null_ptr(), Layout::new::<[u8; 8]>(), 64)
                .expect("Could not grow to 64 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 64);

            alloc
                .grow_in_place(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>(), 65)
                .expect_err("Could grow to 65 bytes in place");

            let memory = alloc
                .grow(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>(), 65)
                .expect("Could not grow to 65 bytes");

            alloc
                .grow(memory.as_non_null_ptr(), Layout::new::<[u8; 100]>(), 512)
                .expect_err("Could grow to 512 bytes");

            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 100]>());
        }
    }

    #[test]
    fn shrink() {
        let mut data = [MaybeUninit::new(0); 256];
        let mut alloc = helper::tracker(Chunk::<_, 64>(Region::new(&mut data)));

        let memory = alloc
            .alloc(Layout::new::<[u8; 128]>())
            .expect("Could not allocate 128 bytes");
        assert_eq!(memory.len() % 64, 0);

        let _cannot_shrink_in_place = alloc
            .alloc(Layout::new::<[u8; 128]>())
            .expect("Could not allocate 128 bytes");

        unsafe {
            let len = alloc
                .shrink_in_place(memory.as_non_null_ptr(), Layout::new::<[u8; 128]>(), 100)
                .expect("Could not shrink to 100 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 128);

            let len = alloc
                .shrink_in_place(memory.as_non_null_ptr(), Layout::new::<[u8; 100]>(), 65)
                .expect("Could not shrink to 65 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 128);

            alloc
                .shrink_in_place(memory.as_non_null_ptr(), Layout::new::<[u8; 65]>(), 64)
                .expect_err("Could shrink to 64 bytes in place");

            let memory = alloc
                .shrink(memory.as_non_null_ptr(), Layout::new::<[u8; 100]>(), 64)
                .expect("Could not shrink to 64 bytes");

            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>());
        }
    }
}
