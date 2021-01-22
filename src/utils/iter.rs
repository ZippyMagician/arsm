use std::ptr::{self, NonNull};

// Instead of iterating over references to values, it iterates over owned values.
// This circumvents an issue with the lifetimes of files or Clap-owned user input.
// It was easier to just make a custom iterator than it was to fix the lifetimes.
// This iterator was designed for a very particular use-case, and as such is close
// in speed to std::slice::Iter
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BufIter<T> {
    ptr: NonNull<T>,
    c: usize,
}

impl<T> BufIter<T> {
    pub fn new(slice: &mut [T]) -> Self
    where
        T: std::fmt::Debug,
    {
        Self {
            ptr: NonNull::new(slice.as_mut_ptr()).expect(&*format!(
                "Could not construct iterator over STDIN {:?}",
                slice
            )),
            c: slice.len(),
        }
    }
}

impl<T> Iterator for BufIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.c == 0 {
            None
        } else {
            self.c -= 1;
            let out;
            // Safety: Bounds won't be reached yet, as self.c check ensures it. Therefore, this is unsafe is safe
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
