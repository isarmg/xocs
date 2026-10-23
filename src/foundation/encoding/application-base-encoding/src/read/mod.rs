//! Implementations of `io::Read` to transparently decode base64.
mod decoder;
pub use self::decoder::DecoderReader;

#[path = "../../../../../../test/foundation/encoding/application-base-encoding/unit/src/read/decoder_tests.rs"]
#[cfg(test)]
mod decoder_tests;
