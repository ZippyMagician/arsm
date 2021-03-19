/// Describes the size of the Number to be written or read
pub trait Size {
    fn len() -> usize;
}

macro_rules! impl_size_for {
    ($($type:ty: $val:expr),*) => {
        $(
            impl Size for $type {
                #[inline(always)]
                fn len() -> usize {
                    $val
                }
            }
        )*
    }
}

impl_size_for!(u8: 1, i16: 2, i32: 4, usize: 8);

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
