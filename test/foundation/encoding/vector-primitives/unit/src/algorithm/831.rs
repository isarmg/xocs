#[cfg(test)]
pub fn i8_lt(a: i8, b: i8) -> u8 {
    if a < b {
        0xff
    } else {
        0x00
    }
}
