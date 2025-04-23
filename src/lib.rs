#![no_std]

use core::ops::Deref;
use core::ops::DerefMut;
use core::ops::Index;
use core::ops::IndexMut;
use core::ptr;

/// wrapper around `*mut [T]` that allows iterating over the values and indexing on pointers.
#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct UncookedSlice<T> {
    // not public: can't expose unsound API, as it would cause UB without the use of unsafe in
    // for loop and indexing. use UncookedSlice::new to construct instead.
    inner: *mut [T],
}

impl<T> UncookedSlice<T> {
    /// # Safety
    ///
    /// Indexing the resulting type with a usize that would cause the pointer value
    /// to wrap around the address space will cause undefined behavior. See [ptr::add]
    /// for details.
    ///
    /// [ptr::add]: https://doc.rust-lang.org/std/primitive.pointer.html#method.add
    ///
    /// ^ Pro tip: you don't usually need to care about this unless your index calculation
    /// is horribly wrong.
    ///
    /// You also need to make sure everything is initialized. Or iterating over the values
    /// will cause UB whenever you encounter an uninitialized one.
    pub const unsafe fn new(ptr: *mut [T]) -> Self {
        UncookedSlice { inner: ptr }
    }

    pub const fn inner(self) -> *mut [T] {
        self.inner
    }
}

impl<T> Index<usize> for UncookedSlice<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        unsafe { &*self.inner.cast::<T>().add(index) }
    }
}

impl<T> IndexMut<usize> for UncookedSlice<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.inner.cast::<T>().add(index) }
    }
}

impl<T> Deref for UncookedSlice<T> {
    type Target = *mut [T];

    fn deref(&self) -> &*mut [T] {
        &self.inner
    }
}

impl<T> DerefMut for UncookedSlice<T> {
    fn deref_mut(&mut self) -> &mut *mut [T] {
        &mut self.inner
    }
}

impl<T: Copy> Iterator for UncookedSlice<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let old_len = match self.inner.len() {
            0 => return None,
            len => len,
        };

        let old_ptr = self.inner.cast::<T>();

        let new_len = old_len - 1;
        let new_ptr = unsafe { old_ptr.add(1) };

        let new_slice = ptr::slice_from_raw_parts_mut(new_ptr, new_len);
        self.inner = new_slice;

        Some(unsafe { *old_ptr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterable() {
        let mut data = [0, 1, 2, 3, 4, 5];
        let ptr = &raw mut data[..];
        let uncooked = unsafe { UncookedSlice::new(ptr) };

        let mut buf = [0i32; 6];
        for (i, item) in uncooked.enumerate() {
            buf[i] = item;
        }

        let uncooked2 = unsafe { UncookedSlice::new(ptr) };
        assert_eq!(uncooked, uncooked2);
        assert_eq!(buf, data);
    }

    #[test]
    fn test_deref_raw_slice() {
        let mut data = [0, 1, 2, 3, 4, 5];
        let ptr = &raw mut data[..];
        let uncooked = unsafe { UncookedSlice::new(ptr) };

        let len = uncooked.len();

        assert_eq!(uncooked.inner.len(), len);
    }

    #[test]
    #[allow(unconditional_panic)]
    #[allow(clippy::out_of_bounds_indexing)]
    fn test_index() {
        extern crate std;
        let mut data = [0, 1, 2, 3, 4, 5];
        let ptr = &raw mut data[..];
        let uncooked = unsafe { UncookedSlice::new(ptr) };

        assert_eq!(data[0], 0);
        assert_eq!(data[1], 1);
        assert_eq!(data[2], 2);
        assert_eq!(data[3], 3);
        assert_eq!(data[4], 4);
        assert_eq!(data[5], 5);

        let catch_unwind = std::panic::catch_unwind(|| data[6]);
        assert!(catch_unwind.is_err())
    }

    #[test]
    #[allow(unconditional_panic)]
    #[allow(clippy::out_of_bounds_indexing)]
    fn test_index_write() {
        extern crate std;
        let mut data = [0, 1, 2, 3, 4, 5];
        let ptr = &raw mut data[..];
        let mut uncooked = unsafe { UncookedSlice::new(ptr) };

        uncooked[0] = 1;
        assert_eq!(uncooked[0], 1);
        uncooked[0] = uncooked[5];
        assert_eq!(uncooked[0], 5);

        let catch_unwind = std::panic::catch_unwind(|| data[6]);
        assert!(catch_unwind.is_err())
    }
}
