use std::fmt;

pub struct Address {
    pub street: String,
    pub house_number: u16,
    pub code: u32,
    pub town: String,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}, {} {}",
            self.street, self.house_number, self.code, self.town
        )
    }
}
