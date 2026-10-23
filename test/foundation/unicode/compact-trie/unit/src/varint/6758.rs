/// A secondary implementation that separates the latent value while computing the varint.
#[cfg(test)]
pub(crate) const fn write_varint_reference(
    value: usize,
) -> ConstArrayBuilder<MAX_VARINT_LENGTH, u8> {
    let mut result = [0; MAX_VARINT_LENGTH];
    if value < 32 {
        result[0] = value as u8;
        return ConstArrayBuilder::from_manual_slice(result, 0, 1);
    }
    result[0] = 32;
    let mut latent = 32;
    let mut steps = 2;
    loop {
        let next_latent = (latent << 7) + 32;
        if value < next_latent || next_latent == latent {
            break;
        }
        latent = next_latent;
        steps += 1;
    }
    let mut value = value - latent;
    let mut i = steps;
    while i > 0 {
        i -= 1;
        result[i] |= (value as u8) & 0b01111111;
        value >>= 7;
        if i > 0 && i < steps - 1 {
            result[i] |= 0b10000000;
        }
    }
    // The bytes are from 0 to `steps`.
    ConstArrayBuilder::from_manual_slice(result, 0, steps)
}
