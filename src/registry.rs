use crate::utils;

#[derive(Debug, PartialEq)]
pub struct Register {
    buf: [u8; 10],
}

#[derive(Debug, PartialEq)]
pub enum Position {
    Lower,

    Upper,
}

impl Register {
    pub fn init() -> Self {
        Self { buf: [0; 10] }
    }

    pub fn read_8(&mut self, key: char, half: Position) -> u8 {
        self.from_key(key)[if half == Position::Upper { 1 } else { 0 }]
    }

    pub fn read_16(&mut self, key: char) -> i16 {
        let pos = self.from_key_pos(key);
        utils::read_from_mem_16(self.buf.as_mut(), pos)
    }

    pub fn read_32(&mut self, left: char, right: char) -> i32 {
        let bytes = utils::join_slices(&self.from_key(left), &self.from_key(right));

        i32::from_ne_bytes(bytes)
    }

    pub fn write_8(&mut self, key: char, half: Position, val: u8) {
        let pos = self.from_key_pos(key);

        if half == Position::Upper {
            self.buf[pos + 1] = val;
        } else {
            self.buf[pos] = val;
        }
    }

    pub fn write_16(&mut self, key: char, val: i16) {
        let pos = self.from_key_pos(key);
        utils::write_to_mem_16(self.buf.as_mut(), pos, val);
    }

    pub fn write_32(&mut self, left: char, right: char, val: i32) {
        let slice = val.to_ne_bytes();
        let lpos = self.from_key_pos(left);
        let rpos = self.from_key_pos(right);

        utils::write(self.buf.as_mut(), lpos, &slice[0..2]);
        utils::write(self.buf.as_mut(), rpos, &slice[2..4]);
    }

    fn from_key(&self, key: char) -> [u8; 2] {
        let pos = "abcdeABCDE".find(key).unwrap() % 5;
        [self.buf[pos * 2], self.buf[pos * 2 + 1]]
    }

    fn from_key_pos(&self, key: char) -> usize {
        let pos = "abcdeABCDE".find(key).unwrap() % 5;
        pos * 2
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn test_modifications() {
        let mut mem = Register::init();

        mem.write_8('e', Position::Lower, 13);
        mem.write_8('e', Position::Upper, 56);
        mem.write_16('a', 1034);
        mem.write_32('c', 'd', 34000);

        assert_eq!(mem.read_16('a'), 1034);
        assert_eq!(mem.read_16('b'), 0);
        assert_eq!(mem.read_32('c', 'd'), 34000);
        assert_eq!(mem.read_8('e', Position::Lower), 13);
        assert_eq!(mem.read_8('e', Position::Upper), 56);

        assert_eq!(mem.buf.len(), 10);
    }
}
