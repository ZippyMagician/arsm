#![allow(dead_code)]

use std::convert::AsMut;
use std::fs;
use std::ptr;

// Safety: Only works when the size of <A> divided by the size of <B> is the length of the slice.
pub fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

// Joines two slices of length 2 → slice of length 4
pub fn join_slices<'a>(left: &'a [u8], right: &'a [u8]) -> [u8; 4] {
    [left[0], left[1], right[0], right[1]]
}

// Safety: Same safety requirements as std::ptr::write_bytes
pub unsafe fn write_to_mem_8(mem: &mut [u8], pos: usize, val: u8) {
    mem.rotate_left(pos);
    ptr::write_bytes(mem.as_mut_ptr(), val, 1);
    mem.rotate_right(pos);
}

// Safety: Same safety requirements as std::ptr::write_bytes
pub unsafe fn write_to_mem_16(mem: &mut [u8], pos: usize, val: i16) {
    let ne_bytes = val.to_ne_bytes();
    let ptr = mem.as_mut_ptr();
    mem.rotate_left(pos);
    ptr::write_bytes(ptr, ne_bytes[0], 1);
    mem.rotate_left(1);
    ptr::write_bytes(ptr, ne_bytes[1], 1);
    mem.rotate_right(pos + 1);
}

// Safety: Same safety requirements as std::ptr::write_bytes
pub unsafe fn write_to_mem_32(mem: &mut [u8], pos: usize, val: i32) {
    let ne_bytes = val.to_ne_bytes();
    let ptr = mem.as_mut_ptr();
    mem.rotate_left(pos);
    ptr::write_bytes(ptr, ne_bytes[0], 1);
    mem.rotate_left(1);
    ptr::write_bytes(ptr, ne_bytes[1], 1);
    mem.rotate_left(1);
    ptr::write_bytes(ptr, ne_bytes[2], 1);
    mem.rotate_left(1);
    ptr::write_bytes(ptr, ne_bytes[3], 1);
    mem.rotate_right(pos + 3);
}

// Safety: Same safety requirements as std::ptr::read
pub unsafe fn read_from_mem_8(mem: &mut [u8], pos: usize) -> u8 {
    mem.rotate_left(pos);
    let ret = ptr::read(mem.as_mut_ptr());
    mem.rotate_right(pos);

    ret
}

// Safety: Same safety requirements as std::ptr::read
pub unsafe fn read_from_mem_16(mem: &mut [u8], pos: usize) -> i16 {
    mem.rotate_left(pos);
    let a = ptr::read(mem.as_mut_ptr());
    mem.rotate_left(1);
    let b = ptr::read(mem.as_mut_ptr());
    mem.rotate_right(pos + 1);

    i16::from_ne_bytes([a, b])
}

// Safety: Same safety requirements as std::ptr::read
pub unsafe fn read_from_mem_32(mem: &mut [u8], pos: usize) -> i32 {
    let ptr = mem.as_mut_ptr();
    mem.rotate_left(pos);
    let a = ptr::read(ptr);
    mem.rotate_left(1);
    let b = ptr::read(ptr);
    mem.rotate_left(1);
    let c = ptr::read(ptr);
    mem.rotate_left(1);
    let d = ptr::read(ptr);
    mem.rotate_right(pos + 3);

    i32::from_ne_bytes([a, b, c, d])
}

// Read file and return the result
pub fn read_file<'b>(path: &'_ str) -> std::io::Result<String> {
    fs::read_to_string(path)
}

#[cfg(test)]
mod mem_tests {
    use super::*;

    #[test]
    fn test_read_write() {
        let mut mem = vec![0u8; 16];
        let mem_slice = mem.as_mut_slice();

        unsafe {
            write_to_mem_8(mem_slice, 0, 56);
            write_to_mem_16(mem_slice, 2, 467);
            write_to_mem_32(mem_slice, 5, 567_735);

            assert_eq!(read_from_mem_32(mem_slice, 5), 567_735);
            assert_eq!(read_from_mem_8(mem_slice, 0), 56);
            assert_eq!(read_from_mem_16(mem_slice, 2), 467);
            assert_eq!(read_from_mem_8(mem_slice, 4), 0);
        }
    }
}
