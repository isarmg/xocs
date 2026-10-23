#[cfg(test)]
impl Byte for u8 {
    fn to_char(self) -> char {
        assert!(self.is_ascii());
        char::from(self)
    }
}
