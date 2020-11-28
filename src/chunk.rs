use crate::{helper::AllocInit, Owns, ReallocateInPlace};
use core::{
    alloc::{AllocError, AllocRef, Layout},
    ptr::NonNull,
};

/// Allocate memory with a multiple size of the provided chunk size.
///
/// # Examples
///
/// ```rust
/// #![feature(allocator_api, slice_ptr_len)]
///
/// use alloc_compose::Chunk;
/// use std::alloc::{AllocRef, Layout, System};
///
/// let mut alloc = Chunk::<_, 64>(System);
/// let ptr = alloc.alloc(Layout::new::<[u8; 16]>())?;
/// assert_eq!(ptr.len() % 64, 0);
/// assert!(ptr.len() >= 64);
/// # Ok::<(), core::alloc::AllocError>(())
/// ```
///
/// When growing or shrinking the memory, `Chunk` will try to alter
/// the memory in place before delegating to the underlying allocator.
///
/// ```rust
/// #![feature(slice_ptr_get)]
/// # #![feature(allocator_api, slice_ptr_len)]
/// # use alloc_compose::Chunk;
/// # use std::{alloc::{AllocRef, Layout, System}};
/// # let mut alloc = Chunk::<_, 64>(System);
/// # let ptr = alloc.alloc(Layout::new::<[u8; 16]>())?;
///
/// let new_ptr = unsafe {
///     alloc.grow(
///         ptr.as_non_null_ptr(),
///         Layout::new::<[u8; 16]>(),
///         Layout::new::<[u8; 24]>(),
///     )?
/// };
///
/// assert_eq!(ptr, new_ptr);
/// # Ok::<(), core::alloc::AllocError>(())
/// ```
///
/// This can be enforced by using [`ReallocateInPlace::grow_in_place`].
///
/// ```rust
/// # #![feature(allocator_api, slice_ptr_len, slice_ptr_get)]
/// # use alloc_compose::Chunk;
/// # use std::{alloc::{AllocRef, Layout, System}};
/// # let mut alloc = Chunk::<_, 64>(System);
/// # let ptr = alloc.alloc(Layout::new::<[u8; 24]>())?;
/// use alloc_compose::ReallocateInPlace;
///
/// let len = unsafe {
///     alloc.grow_in_place(
///         ptr.as_non_null_ptr(),
///         Layout::new::<[u8; 24]>(),
///         Layout::new::<[u8; 32]>(),
///     )?
/// };
///
/// assert_eq!(len % 64, 0);
/// assert!(len >= 64);
/// # Ok::<(), core::alloc::AllocError>(())
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
            impl<A> SizeIsPowerOfTwo for Chunk<A, { usize::pow(2, $N) }> {}
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
    fn round_up(size: usize) -> Result<usize, AllocError> {
        Ok((size.checked_add(SIZE).ok_or(AllocError)? - 1) & !(SIZE - 1))
    }

    unsafe fn round_up_unchecked(size: usize) -> usize {
        let new_size = (size.wrapping_add(SIZE) - 1) & !(SIZE - 1);
        debug_assert_eq!(new_size, Self::round_up(size).unwrap());
        new_size
    }

    const fn round_down(size: usize) -> usize {
        size & !(SIZE - 1)
    }

    const fn round_down_ptr_len(ptr: NonNull<[u8]>) -> NonNull<[u8]> {
        NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), Self::round_down(ptr.len()))
    }

    #[inline]
    fn alloc_impl(
        layout: Layout,
        alloc: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocError>,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let new_size = Self::round_up(layout.size())?;
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

        alloc(new_layout).map(Self::round_down_ptr_len)
    }

    #[inline]
    unsafe fn grow_impl(
        old_ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        init: AllocInit,
        grow: impl FnOnce(NonNull<u8>, Layout, Layout) -> Result<NonNull<[u8]>, AllocError>,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_size = old_layout.size();
        let current_size = Self::round_up_unchecked(old_size);
        let new_size = new_layout.size();
        if new_layout.align() <= old_layout.align() && new_size <= current_size {
            let ptr = NonNull::slice_from_raw_parts(old_ptr, current_size);
            init.init_offset(ptr, old_size);
            return Ok(ptr);
        }

        grow(
            old_ptr,
            Layout::from_size_align_unchecked(current_size, old_layout.align()),
            Layout::from_size_align_unchecked(Self::round_up(new_size)?, new_layout.align()),
        )
        .map(Self::round_down_ptr_len)
    }

    #[inline]
    unsafe fn shrink_impl(
        old_ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        shrink: impl FnOnce(NonNull<u8>, Layout, Layout) -> Result<NonNull<[u8]>, AllocError>,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let current_size = Self::round_up_unchecked(old_layout.size());
        let new_size = new_layout.size();
        if new_layout.align() <= old_layout.align() && new_layout.size() > current_size - SIZE {
            return Ok(NonNull::slice_from_raw_parts(old_ptr, current_size));
        }

        shrink(
            old_ptr,
            old_layout,
            Layout::from_size_align_unchecked(
                Self::round_up_unchecked(new_size),
                new_layout.align(),
            ),
        )
        .map(Self::round_down_ptr_len)
    }
}

unsafe impl<A: AllocRef, const SIZE: usize> AllocRef for Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    impl_alloc_ref!(0);

    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
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

// unsafe impl<A: AllocateAll, const SIZE: usize> AllocateAll for Chunk<A, SIZE>
// where
//     Self: SizeIsPowerOfTwo,
// {
//     impl_alloc_all!(0);
// }

unsafe impl<A, const SIZE: usize> ReallocateInPlace for Chunk<A, SIZE>
where
    Self: SizeIsPowerOfTwo,
{
    impl_realloc_in_place_spec!(0);
}

unsafe impl<A: ReallocateInPlace, const SIZE: usize> ReallocateInPlace for Chunk<A, SIZE>
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
    use crate::{helper::tracker, ReallocateInPlace};
    use alloc::alloc::Global;
    use core::alloc::{AllocRef, Layout};

    #[test]
    fn alloc() {
        let alloc = Chunk::<_, 64>(tracker(Global));
        let memory = alloc
            .alloc(Layout::new::<[u8; 2]>())
            .expect("Could not allocate 64 bytes");
        assert_eq!(memory.len() % 64, 0);
        assert!(memory.len() >= 64);

        unsafe {
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<u8>());
        }
    }

    #[test]
    fn dealloc() {
        let alloc = Chunk::<_, 64>(tracker(Global));

        unsafe {
            let memory = alloc
                .alloc(Layout::new::<[u8; 4]>())
                .expect("Could not allocate 4 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 4]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 8]>())
                .expect("Could not allocate 8 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 8]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 32]>())
                .expect("Could not allocate 32 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 32]>());

            let memory = alloc
                .alloc(Layout::new::<[u8; 64]>())
                .expect("Could not allocate 64 bytes");
            assert_eq!(memory.len() % 64, 0);
            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>());
        }
    }

    #[test]
    fn grow() {
        let alloc = Chunk::<_, 64>(tracker(Global));

        let memory = alloc
            .alloc(Layout::new::<[u8; 4]>())
            .expect("Could not allocate 4 bytes");
        assert_eq!(memory.len() % 64, 0);

        unsafe {
            let len = alloc
                .grow_in_place(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 4]>(),
                    Layout::new::<[u8; 64]>(),
                )
                .expect("Could not grow to 8 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 64);

            let len = alloc
                .grow_in_place(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 8]>(),
                    Layout::new::<[u8; 64]>(),
                )
                .expect("Could not grow to 64 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 64);

            alloc
                .grow_in_place(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 64]>(),
                    Layout::new::<[u8; 65]>(),
                )
                .expect_err("Could grow to 65 bytes in place");

            let memory = alloc
                .grow(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 64]>(),
                    Layout::new::<[u8; 65]>(),
                )
                .expect("Could not grow to 65 bytes");

            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 65]>());
        }
    }

    #[test]
    fn shrink() {
        let alloc = Chunk::<_, 64>(tracker(Global));

        let memory = alloc
            .alloc(Layout::new::<[u8; 128]>())
            .expect("Could not allocate 128 bytes");
        assert_eq!(memory.len() % 64, 0);

        unsafe {
            let len = alloc
                .shrink_in_place(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 128]>(),
                    Layout::new::<[u8; 100]>(),
                )
                .expect("Could not shrink to 100 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 128);

            let len = alloc
                .shrink_in_place(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 100]>(),
                    Layout::new::<[u8; 65]>(),
                )
                .expect("Could not shrink to 65 bytes");
            assert_eq!(len % 64, 0);
            assert!(len >= 128);

            alloc
                .shrink_in_place(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 65]>(),
                    Layout::new::<[u8; 64]>(),
                )
                .expect_err("Could shrink to 64 bytes in place");

            let memory = alloc
                .shrink(
                    memory.as_non_null_ptr(),
                    Layout::new::<[u8; 128]>(),
                    Layout::new::<[u8; 64]>(),
                )
                .expect("Could not shrink to 64 bytes");

            alloc.dealloc(memory.as_non_null_ptr(), Layout::new::<[u8; 64]>());
        }
    }
}
