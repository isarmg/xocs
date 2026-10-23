//! Implementations of `io::Write` to transparently handle base64.
mod encoder;
mod encoder_string_writer;

pub use self::{
    encoder::EncoderWriter,
    encoder_string_writer::{EncoderStringWriter, StrConsumer},
};

#[path = "../../../../../../test/foundation/encoding/application-base-encoding/unit/src/write/encoder_tests.rs"]
#[cfg(test)]
mod encoder_tests;
