use std::iter::Iterator;
use std::ptr;

// A custom iterator implementation that allows what `.iter()` does not for stdin
// Namely, there were issues to do with the lifetime of the string contents of a read file
// This fixes that, as it explicitly allocates the data onto the heap
// It is also rather fast, making use of std::ptr
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BufIter<T> {
    ptr: *mut T,
    end: *const T,
}

impl<T> BufIter<T> {
    pub fn new(body: &mut [T]) -> Self {
        assert_ne!(std::mem::size_of::<T>(), 0);
        assert!(body.len() <= std::isize::MAX as usize);
        // Safety: Previous conditions ensure this will work and not result in UB
        unsafe {
            let ptr = body.as_mut_ptr();
            Self {
                ptr,
                end: ptr.add(body.len()),
            }
        }
    }
}

impl<T> Iterator for BufIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // Safety: meets all requirements of pointer::offset_from, therefore safe
        if unsafe { self.ptr.offset_from(self.end) } == 0 {
            None
        } else {
            let old = self.ptr;
            // Safety: Wrapping code ensures this cannot overflow or read a null pointer
            unsafe {
                self.ptr = self.ptr.offset(1);
                Some(ptr::read(old))
            }
        }
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut v = vec![1, 2, 3, 5, 3, 9, 6];
        let mut iter = BufIter::new(v.as_mut_slice());

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        iter.next();
        assert_eq!(iter.next(), Some(5))
    }

    #[test]
    fn test_empty() {
        let mut iter: BufIter<u8> = BufIter::new(&mut []);
        assert!(iter.next().is_none());
    }
}
