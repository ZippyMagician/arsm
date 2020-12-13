use std::iter::Iterator;
use std::ptr;

// A custom iterator implementation that allows what `.iter()` does not for stdin
// Namely, there were issues to do with the lifetime of the string contents of a read file
// This fixes that, as it explicitly allocates the data onto the heap
// It is also rather fast, making use of std::ptr
#[derive(Debug, PartialEq)]
pub struct BufIter {
    inside: Box<[u8]>,
    c: usize,
}

impl BufIter {
    pub fn new(body: Vec<u8>) -> Self {
        Self {
            inside: body.into_boxed_slice(),
            c: 0,
        }
    }
}

impl Iterator for BufIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.c == self.inside.len() {
            None
        } else {
            let p = self.inside.as_mut_ptr();
            let e;
            unsafe {
                e = ptr::read(p);
                ptr::copy(p.offset(1), p, self.inside.len() - 1);
                self.c += 1;
            }

            Some(e)
        }
    }
}
