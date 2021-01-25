// There are some unused functions that will be used eventually
#![allow(dead_code)]

use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ops::Range,
};
use std::{fmt, ptr};

use super::consts::{OFFSET, REGISTRY_OFFSET, U8_ALIGN};
use super::traits::*;

#[derive(PartialEq)]
pub struct Memory {
    mem: *mut u8,
    pub size: usize,
    pub s_size: usize,
    pub s_len: usize,
}

// GENERAL PURPOSE
impl Memory {
    pub fn init(mem_size: usize, s_size: usize) -> Self {
        assert!(
            mem_size - OFFSET >= s_size,
            "invalid size parameters provided"
        );
        // Safety: Above check ensures this is valid
        let mem = unsafe {
            alloc_zeroed(
                Layout::from_size_align(mem_size, U8_ALIGN).expect("Could not allocate memory"),
            )
        };

        Self {
            mem,
            size: mem_size,
            s_size,
            s_len: 0,
        }
    }

    #[inline]
    fn write(&mut self, loc: usize, element: u8) {
        // Safety: All calls to this function are checked (private)
        unsafe {
            ptr::write(self.mem.add(loc), element);
        }
    }

    #[inline]
    fn read(&self, loc: usize) -> u8 {
        unsafe { ptr::read(self.mem.add(loc)) }
    }

    pub fn resize_stack(&mut self, s_size: usize) {
        if s_size > self.s_size {
            unsafe {
                ptr::write_bytes(self.mem.add(self.s_size), 0, s_size - self.s_size);
            }
        }

        self.s_size = s_size;
        self.s_len = self.s_len.min(s_size);
    }

    #[inline]
    pub fn clear(&mut self) {
        // Safety: since it is guaranteed to be a non-null pointer of self.size length,
        // this is safe
        unsafe {
            ptr::write_bytes(self.mem, 0, self.size);
        }
    }

    #[inline]
    pub fn memory_len(&self) -> usize {
        self.size - self.s_size - OFFSET
    }

    #[inline]
    // Safety: Range must be ascending and fit within 0..=N, where N is the size of Memory
    pub unsafe fn read_range(&self, range: Range<usize>) -> &[u8] {
        assert!(range.start > 0 && range.end < self.size);
        std::slice::from_raw_parts(self.mem.add(range.start), range.end - range.start)
    }

    #[inline]
    pub fn write_range(&mut self, range: Range<usize>, vals: &[u8]) {
        assert!(range.start > 0 && range.end < self.size);
        for (i, &val) in range.zip(vals.iter()) {
            self.write(i, val);
        }
    }
}

// FLAGS
impl Memory {
    // cmp flags (0, eq, g, l, unused, unused, unused)
    #[inline]
    pub fn flag_write_cmp(&mut self, ind: u8) {
        let mask = 1 << ind;
        let num = self.read(REGISTRY_OFFSET);

        self.write(REGISTRY_OFFSET, mask | num);
    }

    #[inline]
    pub fn flag_write_whole_cmp(&mut self, val: u8) {
        self.write(REGISTRY_OFFSET, val);
    }

    #[inline]
    pub fn flag_reset_cmp(&mut self) {
        self.write(REGISTRY_OFFSET, 0);
    }

    #[inline]
    pub fn flag_read_cmp(&self, ind: u8) -> bool {
        let mask = 1 << ind;
        self.read(REGISTRY_OFFSET) & mask != 0
    }
}

// REGISTRY
impl Memory {
    pub fn r_write<N: Num, P: Position>(&mut self, keys: &P, element: &N) {
        let bytes = element.get_bytes();
        match keys.indexes() {
            (Some(a), None, None) => {
                self.write(a, bytes[0]);
                self.write(a + 1, bytes[1]);
            }

            (Some(a), None, Some(side)) => {
                if Pos::Upper == side {
                    self.write(a + 1, bytes[0]);
                } else {
                    self.write(a, bytes[0]);
                }
            }

            (Some(a), Some(b), None) => {
                self.write(a, bytes[0]);
                self.write(a + 1, bytes[1]);
                self.write(b, bytes[2]);
                self.write(b + 1, bytes[3]);
            }

            _ => {}
        }
    }

    pub fn r_read<N: Num>(&self, keys: &dyn Position) -> N {
        match keys.indexes() {
            (Some(a), None, None) => N::from_bytes(&[self.read(a), self.read(a + 1)]),

            (Some(a), None, Some(side)) => N::from_bytes(&[if Pos::Upper == side {
                self.read(a + 1)
            } else {
                self.read(a)
            }]),

            (Some(a), Some(b), None) => N::from_bytes(&[
                self.read(a),
                self.read(a + 1),
                self.read(b),
                self.read(b + 1),
            ]),

            _ => panic!("Something went wrong"),
        }
    }
}

// STACK
impl Memory {
    pub fn s_push<N: Num>(&mut self, element: &N) {
        let bytes = element.get_bytes();
        for byte in &bytes {
            if self.s_len == self.s_size {
                panic!("s is full. Cannot fit size {} element.", bytes.len());
            } else {
                self.write(OFFSET + self.s_len, *byte);
                self.s_len += 1;
            }
        }
    }

    pub fn s_pop_8(&mut self) -> Option<u8> {
        if self.s_len == 0 {
            None
        } else {
            self.s_len -= 1;
            let num = self.read(OFFSET + self.s_len);
            // Let's be sanitary and zero the value
            unsafe {
                ptr::write(self.mem.add(OFFSET + self.s_len), 0);
            }

            Some(num)
        }
    }

    pub fn s_pop_16(&mut self) -> Option<i16> {
        if self.s_len < 2 {
            None
        } else {
            self.s_len -= 2;
            let num = i16::from_ne_bytes([
                self.read(OFFSET + self.s_len),
                self.read(OFFSET + 1 + self.s_len),
            ]);
            unsafe {
                ptr::write_bytes(self.mem.add(OFFSET + self.s_len), 0, 2);
            }

            Some(num)
        }
    }

    pub fn s_pop_32(&mut self) -> Option<i32> {
        if self.s_len < 4 {
            None
        } else {
            self.s_len -= 4;
            let num = i32::from_ne_bytes([
                self.read(OFFSET + self.s_len),
                self.read(OFFSET + 1 + self.s_len),
                self.read(OFFSET + 2 + self.s_len),
                self.read(OFFSET + 3 + self.s_len),
            ]);
            unsafe {
                ptr::write_bytes(self.mem.add(OFFSET + self.s_len), 0, 4);
            }

            Some(num)
        }
    }
}

// MEMORY
impl Memory {
    pub fn m_write<N: Num>(&mut self, pos: usize, val: &N) {
        let start = OFFSET + self.s_size;
        let bytes = val.get_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            if start + pos + i >= self.size {
                panic!(
                    "Cannot fit {} byte number into memory of size {} at point {}",
                    bytes.len(),
                    self.size,
                    start + pos
                );
            } else {
                self.write(start + pos + i, *byte);
            }
        }
    }

    pub fn m_read<N: Num>(&self, pos: usize) -> N {
        let start = OFFSET + self.s_size;

        match N::len() {
            1 => N::from_bytes(&[self.read(start + pos)]),

            2 => N::from_bytes(&[self.read(start + pos), self.read(start + pos + 1)]),

            4 => N::from_bytes(&[
                self.read(start + pos),
                self.read(start + pos + 1),
                self.read(start + pos + 2),
                self.read(start + pos + 3),
            ]),

            _ => panic!("Something went wrong"),
        }
    }
}

impl std::ops::Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.mem,
                Layout::from_size_align_unchecked(self.size, U8_ALIGN),
            );
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("mem", &unsafe {
                std::slice::from_raw_parts(self.mem, self.size)
            })
            .field("size", &self.size)
            .field("stack_size", &self.s_size)
            .field("stack_len", &self.s_len)
            .finish()
    }
}

#[cfg(test)]
mod mem_tests {
    use super::*;

    #[test]
    fn test_stack() {
        let mut env = Memory::init(1024, 5);
        env.s_push(&13_u8);
        env.s_push(&128935_i32);

        assert_eq!(env.s_pop_32(), Some(128935));

        env.s_push(&345_i16);
        env.s_push(&68_u8);
        env.s_push(&31_u8);

        assert_eq!(env.s_pop_8(), Some(31));
        assert_eq!(env.s_pop_8(), Some(68));
        assert_eq!(env.s_pop_16(), Some(345));
        assert_eq!(env.s_pop_8(), Some(13));
    }

    #[test]
    fn test_registry() {
        let mut env = Memory::init(1024, 10);
        env.r_write(&'e', &276_i16);
        env.r_write(&('a', 'b'), &128935_i32);
        env.r_write(&('c', Pos::Upper), &7_u8);

        assert_eq!(env.r_read::<i32>(&('a', 'b')), 128935);
        assert_eq!(env.r_read::<u8>(&('c', Pos::Upper)), 7);
        assert_eq!(env.r_read::<i16>(&'e'), 276);
    }

    #[test]
    fn test_memory() {
        let mut env = Memory::init(1024, 0);
        env.m_write(0, &15_u8);
        env.m_write(5, &1056_i16);
        env.m_write(3, &18_u8);
        env.m_write(100, &-65412_i32);

        assert_eq!(env.m_read::<i16>(5), 1056);
        assert_eq!(env.m_read::<i32>(100), -65412);
        assert_eq!(env.m_read::<u8>(0), 15);
        assert_eq!(env.m_read::<u8>(3), 18);
    }

    #[test]
    fn test_together() {
        let mut env = Memory::init(1024, 5);
        env.s_push(&13_u8);
        env.r_write(&'e', &1342_i16);
        assert_eq!(env.s_pop_8(), Some(13));
        assert_eq!(env.r_read::<i16>(&'e'), 1342);
    }

    #[test]
    fn test_flags() {
        let mut env = Memory::init(20, 0);
        env.flag_write_cmp(2);
        env.flag_write_cmp(4);

        assert!(!env.flag_read_cmp(0));
        assert!(env.flag_read_cmp(4));
        env.flag_reset_cmp();
        assert!(!env.flag_read_cmp(2));
    }
}
