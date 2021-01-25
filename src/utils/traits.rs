// An extension of `num_traits::Num` trait that features some functions I use in
// the parser and memory ops. Almost completely inlined because the functions
// are so small, this will greatly improve performance
pub trait Num: num_traits::Num + num_traits::NumCast {
    fn get_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;

    #[inline]
    fn len() -> usize
    where
        Self: Sized,
    {
        0
    }
}

impl Num for u8 {
    #[inline]
    fn get_bytes(&self) -> Vec<u8> {
        vec![*self]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        bytes[0]
    }

    #[inline]
    fn len() -> usize
    where
        Self: Sized,
    {
        1
    }
}

impl Num for i16 {
    #[inline]
    fn get_bytes(&self) -> Vec<u8> {
        self.to_ne_bytes().into()
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        i16::from_ne_bytes([bytes[0], bytes[1]])
    }

    #[inline]
    fn len() -> usize
    where
        Self: Sized,
    {
        2
    }
}

impl Num for i32 {
    #[inline]
    fn get_bytes(&self) -> Vec<u8> {
        self.to_ne_bytes().into()
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    #[inline]
    fn len() -> usize
    where
        Self: Sized,
    {
        4
    }
}

impl Num for usize {
    #[inline]
    fn get_bytes(&self) -> Vec<u8> {
        self.to_ne_bytes().into()
    }

    #[inline]
    fn from_bytes(_: &[u8]) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    #[inline]
    fn len() -> usize
    where
        Self: Sized,
    {
        8
    }
}

type Location = (Option<usize>, Option<usize>, Option<Pos>);

pub trait Position: std::fmt::Debug {
    fn len(&self) -> usize;

    fn indexes(&self) -> Location;
}

impl Position for char {
    fn len(&self) -> usize {
        1
    }

    fn indexes(&self) -> Location {
        (Some("abcde".find(*self).unwrap() * 2), None, None)
    }
}

impl Position for (char, Pos) {
    fn len(&self) -> usize {
        1
    }

    fn indexes(&self) -> Location {
        (Some("abcde".find(self.0).unwrap() * 2), None, Some(self.1))
    }
}

impl Position for (char, char) {
    fn len(&self) -> usize {
        2
    }

    fn indexes(&self) -> Location {
        (
            Some("abcde".find(self.0).unwrap() * 2),
            Some("abcde".find(self.1).unwrap() * 2),
            None,
        )
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Pos {
    Lower,
    Upper,
}

pub trait Status {
    fn has_jmp(&self) -> bool;

    fn get_val(&self) -> i32;
}

impl Status for bool {
    #[inline]
    fn has_jmp(&self) -> bool {
        *self
    }

    #[inline]
    fn get_val(&self) -> i32 {
        0
    }
}

impl Status for i32 {
    #[inline]
    fn has_jmp(&self) -> bool {
        false
    }

    #[inline]
    fn get_val(&self) -> i32 {
        *self
    }
}

impl Status for (bool, i32) {
    #[inline]
    fn has_jmp(&self) -> bool {
        self.0
    }

    #[inline]
    fn get_val(&self) -> i32 {
        self.1
    }
}
