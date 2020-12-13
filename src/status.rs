pub trait Status {
    fn has_jmp(&self) -> bool;

    fn get_val(&self) -> i32;
}

impl Status for bool {
    fn has_jmp(&self) -> bool {
        *self
    }

    fn get_val(&self) -> i32 {
        0
    }
}

impl Status for i32 {
    fn has_jmp(&self) -> bool {
        false
    }

    fn get_val(&self) -> i32 {
        *self
    }
}

impl Status for (bool, i32) {
    fn has_jmp(&self) -> bool {
        self.0
    }

    fn get_val(&self) -> i32 {
        self.1
    }
}
