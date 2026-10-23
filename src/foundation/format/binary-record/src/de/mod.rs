use cobs::{decode_in_place, decode_in_place_report};
use serde::Deserialize;

pub(crate) mod deserializer;
pub mod flavors;

use crate::error::{Error, Result};
use deserializer::Deserializer;

/// Deserialize a message of type `T` from a byte slice. The unused portion (if any)
/// of the byte slice is not returned.
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

/// Deserialize a message of type `T` from a cobs-encoded byte slice.
///
/// The unused portion (if any) of the byte slice is not returned.
/// The used portion of the input slice is modified during deserialization (even if an error is returned).
/// Therefore, if this is not desired, pass a clone of the original slice.
pub fn from_bytes_cobs<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let sz = decode_in_place(s).map_err(|_| Error::DeserializeBadEncoding)?;
    from_bytes::<T>(&s[..sz])
}

/// Deserialize a message of type `T` from a cobs-encoded byte slice.
///
/// The unused portion (if any) of the byte slice is returned for further usage.
/// The used portion of the input slice is modified during deserialization (even if an error is returned).
/// Therefore, if this is not desired, pass a clone of the original slice.
pub fn take_from_bytes_cobs<'a, T>(s: &'a mut [u8]) -> Result<(T, &'a mut [u8])>
where
    T: Deserialize<'a>,
{
    let mut report = decode_in_place_report(s).map_err(|_| Error::DeserializeBadEncoding)?;

    // The report does not include terminator bytes. If there is one in the
    // buffer right AFTER the message, also include it.
    if s.get(report.src_used) == Some(&0) {
        report.src_used += 1;
    }

    // First split off the amount used for the "destination", which includes our now
    // decoded message to deserialize
    let (dst_used, dst_unused) = s.split_at_mut(report.dst_used);

    // Then create a slice that includes the unused bytes, but DON'T include the
    // excess bytes that were "shrunk" away from the original message
    let (_unused, src_unused) = dst_unused.split_at_mut(report.src_used - report.dst_used);
    Ok((from_bytes::<T>(dst_used)?, src_unused))
}

/// Deserialize a message of type `T` from a byte slice. The unused portion (if any)
/// of the byte slice is returned for further usage
pub fn take_from_bytes<'a, T>(s: &'a [u8]) -> Result<(T, &'a [u8])>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()?))
}

/// Deserialize a message of type `T` from a [`std::io::Read`].
#[cfg(feature = "use-std")]
pub fn from_io<'a, T, R>(val: (R, &'a mut [u8])) -> Result<(T, (R, &'a mut [u8]))>
where
    T: Deserialize<'a>,
    R: std::io::Read + 'a,
{
    let flavor = flavors::io::io::IOReader::new(val.0, val.1);
    let mut deserializer = Deserializer::from_flavor(flavor);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()?))
}

////////////////////////////////////////////////////////////////////////////////

#[path = "../../../../../../test/foundation/format/binary-record/unit/src/de/mod/test_heapless.rs"]
#[cfg(feature = "heapless")]
#[cfg(test)]
mod test_heapless;

#[path = "../../../../../../test/foundation/format/binary-record/unit/src/de/mod/test_alloc.rs"]
#[cfg(any(feature = "alloc", feature = "use-std"))]
#[cfg(test)]
mod test_alloc;
