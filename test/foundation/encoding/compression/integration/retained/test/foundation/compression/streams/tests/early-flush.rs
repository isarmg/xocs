use std::io::{Read, Write};

use blog_encoding_compression::streams::read::GzDecoder;
use blog_encoding_compression::streams::write::GzEncoder;

#[test]
fn smoke() {
    let mut w = GzEncoder::new(Vec::new(), blog_encoding_compression::streams::Compression::default());
    w.flush().unwrap();
    w.write_all(b"hello").unwrap();

    let bytes = w.finish().unwrap();

    let mut r = GzDecoder::new(&bytes[..]);
    let mut s = String::new();
    r.read_to_string(&mut s).unwrap();
    assert_eq!(s, "hello");
}
