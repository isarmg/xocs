pub const fn lookup(lut: &[u8; 16], x: u8) -> u8 {
    if x < 0x80 {
        lut[(x & 0x0f) as usize]
    } else {
        0
    }
}

pub const fn avgr(a: u8, b: u8) -> u8 {
    ((a as u16 + b as u16 + 1) >> 1) as u8
}

#[cfg(test)]
include!("../../../../../test/foundation/encoding/vector-primitives/unit/src/algorithm/220.rs");

#[cfg(test)]
include!("../../../../../test/foundation/encoding/vector-primitives/unit/src/algorithm/831.rs");
