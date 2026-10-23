//! Contracts for the bounded record API used by the maintained compiler.
use crate::{from_bytes, from_bytes_cobs, take_from_bytes, to_slice, to_slice_cobs, Error};
use serde::{Deserialize, Serialize};

#[test]
fn integer_and_utf8_wire_vectors() {
    let mut buffer = [0; 64];
    assert_eq!(
        to_slice(&0xA5C7u16, &mut buffer).unwrap(),
        &[0xC7, 0xCB, 0x02]
    );
    assert_eq!(to_slice(&-1i64, &mut buffer).unwrap(), &[1]);
    assert_eq!(
        to_slice(&i64::MIN, &mut buffer).unwrap(),
        &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]
    );
    assert_eq!(
        to_slice("🦀", &mut buffer).unwrap(),
        &[4, 0xF0, 0x9F, 0xA6, 0x80]
    );
    assert_eq!(
        from_bytes::<&str>(&[4, 0xF0, 0x9F, 0xA6, 0x80]).unwrap(),
        "🦀"
    );
}

#[test]
fn borrowed_records_preserve_payload_and_following_record() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Record<'a> {
        name: &'a str,
        bytes: &'a [u8],
    }
    let input = Record {
        name: "中文",
        bytes: &[0, 1, 0, 255],
    };
    let mut buffer = [0; 64];
    let len = to_slice(&input, &mut buffer).unwrap().len();
    buffer[len] = 42;
    let (decoded, tail): (Record<'_>, _) = take_from_bytes(&buffer[..len + 1]).unwrap();
    assert_eq!(decoded, input);
    assert_eq!(tail, &[42]);
    assert!(decoded.bytes.as_ptr() >= buffer.as_ptr());
    assert!(decoded.bytes.as_ptr() < buffer[len..].as_ptr());
}

#[test]
fn short_buffers_and_malformed_records_return_errors() {
    assert_eq!(
        to_slice("hello", &mut [0; 5]),
        Err(Error::SerializeBufferFull)
    );
    assert_eq!(
        from_bytes::<u64>(&[0x80]),
        Err(Error::DeserializeUnexpectedEnd)
    );
    assert_eq!(
        from_bytes::<&str>(&[1, 0xFF]),
        Err(Error::DeserializeBadUtf8)
    );
    assert_eq!(from_bytes::<bool>(&[2]), Err(Error::DeserializeBadBool));
}

#[test]
fn framed_records_preserve_zero_bytes() {
    let input = [1u8, 0, 2, 0, 255];
    let mut buffer = [0; 16];
    let encoded = to_slice_cobs(&input, &mut buffer).unwrap();
    assert_eq!(encoded, &[2, 1, 2, 2, 2, 255, 0]);
    assert_eq!(from_bytes_cobs::<[u8; 5]>(encoded).unwrap(), input);
}

#[cfg(feature = "alloc")]
#[test]
fn owned_and_bounded_outputs_share_the_record_format() {
    let input = ("中文🦀", u128::MAX, Some(-12345i64));
    let mut buffer = [0; 128];
    assert_eq!(
        crate::to_allocvec(&input).unwrap(),
        to_slice(&input, &mut buffer).unwrap()
    );
    assert_eq!(
        crate::to_allocvec_cobs(&input).unwrap(),
        to_slice_cobs(&input, &mut buffer).unwrap()
    );
}
