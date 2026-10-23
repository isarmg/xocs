//! Adapters between asynchronous readers, byte streams and sinks.
//!
//! [`ReaderStream`] converts an asynchronous reader to a byte stream;
//! [`StreamReader`] performs the reverse conversion. These adapters are useful
//! for transferring streaming HTTP bodies without buffering the complete body.
//!
//! The optional `io-util` feature also provides `SyncIoBridge` for synchronous
//! consumers of asynchronous streams. Run such synchronous work on a blocking
//! worker rather than the runtime's I/O thread.

mod copy_to_bytes;
mod inspect;
mod read_buf;
mod reader_stream;
pub mod simplex;
mod sink_writer;
mod stream_reader;
mod write_all_vectored;

cfg_io_util! {
    mod read_arc;
    pub use self::read_arc::read_exact_arc;

    mod sync_bridge;
    pub use self::sync_bridge::SyncIoBridge;
}

pub use self::copy_to_bytes::CopyToBytes;
pub use self::inspect::{InspectReader, InspectWriter};
pub use self::read_buf::read_buf;
pub use self::reader_stream::ReaderStream;
pub use self::sink_writer::SinkWriter;
pub use self::stream_reader::StreamReader;
pub use self::write_all_vectored::{write_all_vectored, WriteAllVectored};
pub use crate::util::{poll_read_buf, poll_write_buf};
