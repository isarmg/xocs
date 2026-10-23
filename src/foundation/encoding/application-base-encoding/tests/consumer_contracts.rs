//! Contracts shared by authentication, HTTP Basic authorization and WASM embedding.
use blog_encoding_application_base_encoding::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
    write::EncoderWriter,
    DecodeError, DecodeSliceError, Engine,
};
use std::io::{self, Read, Write};
use std::{cell::Cell, rc::Rc};

#[test]
fn rfc_4648_and_product_wire_formats() {
    for (plain, encoded) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
        ("Aladdin:open sesame", "QWxhZGRpbjpvcGVuIHNlc2FtZQ=="),
        ("\0asm\u{1}\0\0\0", "AGFzbQEAAAA="),
    ] {
        assert_eq!(STANDARD.encode(plain), encoded);
        assert_eq!(STANDARD.decode(encoded).unwrap(), plain.as_bytes());
        assert_eq!(URL_SAFE.encode(plain), encoded);
        let unpadded = encoded.trim_end_matches('=');
        assert_eq!(STANDARD_NO_PAD.encode(plain), unpadded);
        assert_eq!(URL_SAFE_NO_PAD.decode(unpadded).unwrap(), plain.as_bytes());
    }
    assert_eq!(STANDARD.encode([0xfb, 0xff]), "+/8=");
    assert_eq!(URL_SAFE.encode([0xfb, 0xff]), "-_8=");
    assert_eq!(URL_SAFE_NO_PAD.encode([0xfb, 0xff]), "-_8");
    // The application stores 32 random bytes in canonical, unpadded URL-safe tokens.
    let token: Vec<u8> = (0..32).collect();
    assert_eq!(
        URL_SAFE_NO_PAD.encode(&token),
        "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"
    );
}

#[test]
fn canonical_tokens_reject_aliases_and_wrong_alphabets() {
    for encoded in [
        "Zg", "Zg=", "Zg===", "Zh==", "Zm9=", "Zg==\n", " Zg==", "-_8=",
    ] {
        assert!(STANDARD.decode(encoded).is_err(), "{encoded:?}");
    }
    for encoded in ["Zg=", "Zg==", "Zh", "Zm9", "Zg\n", "+/8", "-_8="] {
        assert!(URL_SAFE_NO_PAD.decode(encoded).is_err(), "{encoded:?}");
    }
    assert_eq!(
        STANDARD.decode("AA!A"),
        Err(DecodeError::InvalidByte(2, b'!'))
    );
    let mut output = [0xa5; 4];
    assert_eq!(STANDARD.decode_slice("Zm9v", &mut output).unwrap(), 3);
    assert_eq!(output, [b'f', b'o', b'o', 0xa5]);
    assert_eq!(
        STANDARD.decode_slice("Zm9v", &mut output[..2]),
        Err(DecodeSliceError::OutputSliceTooSmall)
    );
}

#[derive(Default)]
struct ShortWriter(Vec<u8>);

impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let len = bytes.len().min(3);
        self.0.extend_from_slice(&bytes[..len]);
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn streaming_authorization_preserves_bytes_across_partial_writes() {
    let plain: Vec<u8> = (0..4097).map(|i| (i % 251) as u8).collect();
    let expected = STANDARD.encode(&plain);
    for chunk_size in [1, 2, 3, 7, 255, 1024] {
        let mut encoder = EncoderWriter::new(ShortWriter::default(), &STANDARD);
        for chunk in plain.chunks(chunk_size) {
            encoder.write_all(chunk).unwrap();
            encoder.flush().unwrap();
        }
        let encoded = encoder.finish().unwrap().0;
        assert_eq!(encoded, expected.as_bytes(), "chunk size {chunk_size}");
        let mut decoder = blog_encoding_application_base_encoding::read::DecoderReader::new(
            encoded.as_slice(),
            &STANDARD,
        );
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, plain);
    }
}

#[derive(Default)]
struct InterruptingWriter {
    bytes: Vec<u8>,
    interrupted: bool,
}

impl Write for InterruptingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.interrupted = !self.interrupted;
        if self.interrupted {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = bytes.len().min(5);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn interrupted_writes_retry_without_losing_or_duplicating_input() {
    let input: Vec<u8> = (0..4097).map(|i| (i % 251) as u8).collect();
    let mut encoder = EncoderWriter::new(InterruptingWriter::default(), &STANDARD);
    // Leave a partial triple before an interrupted write completes it.
    encoder.write_all(&input[..1]).unwrap();
    encoder.write_all(&input[1..]).unwrap();
    let output = encoder.finish().unwrap();
    assert_eq!(output.bytes, STANDARD.encode(input).as_bytes());
}

struct FailingWriter {
    bytes: Vec<u8>,
    remaining: Rc<Cell<usize>>,
}

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining.get() == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let count = bytes.len().min(self.remaining.get());
        self.bytes.extend_from_slice(&bytes[..count]);
        self.remaining.set(self.remaining.get() - count);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn retrying_finish_never_repeats_partial_input_or_padding() {
    for input in [b"f".as_slice(), b"fo"] {
        for accepted_before_failure in 0..4 {
            let remaining = Rc::new(Cell::new(accepted_before_failure));
            let mut encoder = EncoderWriter::new(
                FailingWriter {
                    bytes: Vec::new(),
                    remaining: remaining.clone(),
                },
                &STANDARD,
            );
            encoder.write_all(input).unwrap();
            assert_eq!(
                encoder.finish().err().unwrap().kind(),
                io::ErrorKind::WouldBlock
            );
            remaining.set(usize::MAX);
            let output = encoder.finish().unwrap();
            assert_eq!(output.bytes, STANDARD.encode(input).as_bytes());
        }
    }
}

#[derive(Default)]
struct ZeroWriter {
    calls: Rc<Cell<usize>>,
}

impl Write for ZeroWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        // The guard also makes an incorrect retry loop fail deterministically instead of hanging.
        if self.calls.get() > 1 {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn zero_progress_is_reported_without_spinning() {
    let mut encoder = EncoderWriter::new(ZeroWriter::default(), &STANDARD);
    encoder.write_all(b"f").unwrap();
    assert_eq!(
        encoder.finish().err().unwrap().kind(),
        io::ErrorKind::WriteZero
    );
    assert_eq!(encoder.into_inner().calls.get(), 1);

    let calls = Rc::new(Cell::new(0));
    let mut encoder = EncoderWriter::new(
        ZeroWriter {
            calls: calls.clone(),
        },
        &STANDARD,
    );
    encoder.write_all(b"foo").unwrap();
    // Start counting at flush: write_all accepted the input into the encoder's output buffer.
    calls.set(0);
    assert_eq!(
        encoder.flush().unwrap_err().kind(),
        io::ErrorKind::WriteZero
    );
    assert_eq!(encoder.into_inner().calls.get(), 1);
}

#[cfg(all(
    feature = "simd-unsafe",
    any(
        target_arch = "x86_64",
        all(target_arch = "aarch64", target_feature = "neon")
    )
))]
#[test]
fn accelerated_engine_preserves_scalar_output_errors_and_slice_boundaries() {
    use blog_encoding_application_base_encoding::engine::{general_purpose::NO_PAD, Simd};
    let accelerated = Simd::url_safe(NO_PAD);
    for len in (0..=256).chain([1023, 1024, 1025, 4096, 4097]) {
        let input: Vec<u8> = (0..len).map(|i| ((i * 31 + i / 7) % 256) as u8).collect();
        let encoded = URL_SAFE_NO_PAD.encode(&input);
        assert_eq!(accelerated.encode(&input), encoded);
        let mut output = vec![0xa5; len + 16];
        assert_eq!(
            accelerated.decode_slice(&encoded, &mut output).unwrap(),
            len
        );
        assert_eq!(&output[..len], input);
        assert!(output[len..].iter().all(|&byte| byte == 0xa5));
    }
    let valid = URL_SAFE_NO_PAD.encode([42; 128]).into_bytes();
    for position in 0..valid.len() {
        for invalid in [b'!', b'=', b'/', 0x80, 0xff] {
            let mut encoded = valid.clone();
            encoded[position] = invalid;
            assert_eq!(
                accelerated.decode(&encoded),
                URL_SAFE_NO_PAD.decode(&encoded)
            );
        }
    }
}
