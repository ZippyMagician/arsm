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

    pub fn read_8(&self, key: char, half: Position) -> u8 {
        if half == Position::Upper {
            self.from_key(key)[1]
        } else {
            self.from_key(key)[0]
        }
    }

    pub fn read_16(&self, key: char) -> i16 {
        i16::from_ne_bytes(self.from_key(key))
    }

    pub fn read_32(&self, left: char, right: char) -> i32 {
        let bytes = utils::join_slices(&self.from_key(left), &self.from_key(right));

        i32::from_ne_bytes(bytes)
    }

    pub fn write_8(&mut self, key: char, half: Position, val: u8) {
        let [left, right] = self.from_key_pos(key);

        if half == Position::Upper {
            self.buf[right] = val;
        } else {
            self.buf[left] = val;
        }
    }

    pub fn write_16(&mut self, key: char, val: i16) {
        let ne_bytes = val.to_ne_bytes();
        let [left, right] = self.from_key_pos(key);

        self.buf[left] = ne_bytes[0];
        self.buf[right] = ne_bytes[1];
    }

    pub fn write_32(&mut self, left: char, right: char, val: i32) {
        let slice = val.to_ne_bytes();

        self.write_16(
            left,
            i16::from_ne_bytes(utils::clone_into_array(&slice[0..2])),
        );
        self.write_16(
            right,
            i16::from_ne_bytes(utils::clone_into_array(&slice[2..4])),
        );
    }

    fn from_key(&self, key: char) -> [u8; 2] {
        let pos = "abcdeABCDE".find(key).unwrap() % 5;
        [self.buf[pos * 2], self.buf[pos * 2 + 1]]
    }

    fn from_key_pos(&mut self, key: char) -> [usize; 2] {
        let pos = "abcdeABCDE".find(key).unwrap() % 5;
        [pos * 2, pos * 2 + 1]
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn test_read_write() {
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
    }
}
