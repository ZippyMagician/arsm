use std::convert::Into;

pub struct Status {
    jmp: Option<bool>,

    bal: Option<i32>,
}

impl Status {
    pub fn has_jmp(&self) -> bool {
        self.jmp.unwrap_or(false)
    }

    pub fn get_val(&self) -> i32 {
        self.bal.unwrap_or(0)
    }
}

impl Into<Status> for bool {
    fn into(self) -> Status {
        Status {
            jmp: Some(self),
            bal: None,
        }
    }
}

impl Into<Status> for i32 {
    fn into(self) -> Status {
        Status {
            jmp: None,
            bal: Some(self),
        }
    }
}

impl Into<Status> for (bool, i32) {
    fn into(self) -> Status {
        Status {
            jmp: Some(self.0),
            bal: Some(self.1),
        }
    }
}
