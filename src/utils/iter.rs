use std::ptr::{self, NonNull};

// A custom iterator implementation that allows what `.iter()` does not for stdin
// Namely, there were issues to do with the lifetime of the string contents of a read file
// This fixes that, as it explicitly allocates the data onto the heap
// It is also rather fast, making use of std::ptr
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BufIter<T> {
    ptr: NonNull<T>,
    c: usize,
}

impl<T> BufIter<T> {
    pub fn new(slice: &mut [T]) -> Self {
        Self {
            ptr: NonNull::new(slice.as_mut_ptr()).unwrap(),
            c: slice.len(),
        }
    }
}

impl<T> Iterator for BufIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.c == 0 {
            None
        } else {
            self.c -= 1;
            let out;
            unsafe {
                out = ptr::read(self.ptr.as_ptr());
                self.ptr = NonNull::new_unchecked(self.ptr.as_ptr().offset(1));
            }
            Some(out)
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
