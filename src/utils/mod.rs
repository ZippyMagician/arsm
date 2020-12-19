pub mod consts;
pub mod iter;
pub mod status;
pub mod token;

use std::ptr;

// Joines two slices of length 2 → slice of length 4
pub fn join_slices<'a>(left: &'a [u8], right: &'a [u8]) -> [u8; 4] {
    [left[0], left[1], right[0], right[1]]
}

// Unchecked write (assumes inside index)
pub unsafe fn write(mem: &mut [u8], pos: usize, vals: &[u8]) {
    let mut p = mem.as_mut_ptr().add(pos);
    let len = mem.len();

    for val in vals {
        ptr::copy(p, p, len - pos);
        ptr::write(p, *val);
        p = p.offset(1);
    }
}

// Unchecked read (assumes inside index)
#[inline]
unsafe fn read(mem: &mut [u8], pos: usize) -> u8 {
    ptr::read(mem.as_ptr().add(pos))
}

// Safety: Same safety requirements as std::ptr::write_bytes
pub fn write_to_mem_8(mem: &mut [u8], pos: usize, val: u8) {
    unsafe {
        write(mem, pos, &[val]);
    }
}

// Safety: Same safety requirements as std::ptr::write_bytes
pub fn write_to_mem_16(mem: &mut [u8], pos: usize, val: i16) {
    let ne_bytes = val.to_ne_bytes();
    unsafe {
        write(mem, pos, &ne_bytes);
    }
}

// Safety: Same safety requirements as std::ptr::write_bytes
pub fn write_to_mem_32(mem: &mut [u8], pos: usize, val: i32) {
    let ne_bytes = val.to_ne_bytes();
    unsafe {
        write(mem, pos, &ne_bytes);
    }
}

// Safety: Same safety requirements as std::ptr::read
pub fn read_from_mem_8(mem: &mut [u8], pos: usize) -> u8 {
    unsafe { read(mem, pos) }
}

// Safety: Same safety requirements as std::ptr::read
pub fn read_from_mem_16(mem: &mut [u8], pos: usize) -> i16 {
    unsafe { i16::from_ne_bytes([read(mem, pos), read(mem, pos + 1)]) }
}

// Safety: Same safety requirements as std::ptr::read
pub fn read_from_mem_32(mem: &mut [u8], pos: usize) -> i32 {
    unsafe {
        i32::from_ne_bytes([
            read(mem, pos),
            read(mem, pos + 1),
            read(mem, pos + 2),
            read(mem, pos + 3),
        ])
    }
}

#[cfg(test)]
mod mem_tests {
    use super::*;

    #[test]
    fn test_read_write() {
        let mut mem = vec![0u8; 16];
        let mem_slice = mem.as_mut_slice();

        write_to_mem_8(mem_slice, 15, 3);
        write_to_mem_8(mem_slice, 0, 56);
        write_to_mem_16(mem_slice, 2, 467);
        write_to_mem_32(mem_slice, 5, 567_735);

        assert_eq!(read_from_mem_32(mem_slice, 5), 567_735);
        assert_eq!(read_from_mem_8(mem_slice, 0), 56);
        assert_eq!(read_from_mem_16(mem_slice, 2), 467);
        assert_eq!(read_from_mem_8(mem_slice, 4), 0);
        assert_eq!(read_from_mem_8(mem_slice, 15), 3);
    }
}
