#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../../../../../docs/dependencies/origins/postcard-1.1.3/README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod accumulator;
mod de;

mod error;
pub mod fixint;
mod ser;
mod varint;

pub use de::deserializer::Deserializer;
pub use de::flavors as de_flavors;
pub use de::{from_bytes, from_bytes_cobs, take_from_bytes, take_from_bytes_cobs};
pub use error::{Error, Result};
pub use ser::flavors as ser_flavors;
pub use ser::{serialize_with_flavor, serializer::Serializer, to_extend, to_slice, to_slice_cobs};

#[cfg(feature = "heapless")]
pub use ser::{to_vec, to_vec_cobs};

#[cfg(feature = "use-std")]
pub use ser::{to_io, to_stdvec, to_stdvec_cobs};

#[cfg(feature = "use-std")]
pub use de::from_io;

#[cfg(feature = "alloc")]
pub use ser::{to_allocvec, to_allocvec_cobs};

#[path = "../../../../../test/foundation/format/binary-record/unit/src/lib/test.rs"]
#[cfg(test)]
mod test;

#[path = "../../../../../test/foundation/format/binary-record/unit/src/consumer_tests.rs"]
#[cfg(test)]
mod consumer_tests;
