#[cfg(test)]
unsafe fn byte_is_allowed(byte: u8, f: unsafe fn(bytes: &mut Bytes<'_>)) -> bool {
    let mut slice = [b'_'; 16];
    slice[10] = byte;
    let mut bytes = Bytes::new(&slice);

    f(&mut bytes);

    match bytes.pos() {
        16 => true,
        10 => false,
        x => panic!("unexpected pos: {}", x),
    }
}
