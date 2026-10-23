//! Extensions for HTTP messages in Hyper.
//!
//! This module provides types and utilities that extend the capabilities of HTTP requests and responses
//! in Hyper. Extensions are additional pieces of information or features that can be attached to HTTP
//! messages via the [`http::Extensions`] map, which is
//! accessible through methods like [`http::Request::extensions`] and [`http::Response::extensions`].
//!
//! # What are extensions?
//!
//! Extensions allow Hyper to associate extra metadata or behaviors with HTTP messages, beyond the standard
//! headers and body. These can be used by advanced users and library authors to access protocol-specific
//! features, track original header casing, handle informational responses, and more.
//!
//! # How to access extensions
//!
//! Extensions are stored in the `Extensions` map of a request or response. You can access them using:
//!
//! ```rust
//! # let response = http::Response::new(());
//! if let Some(ext) = response.extensions().get::<http_transport::ext::ReasonPhrase>() {
//!     // use the extension
//! }
//! ```
//!
//! # Extension Groups
//!
//! The extensions in this module can be grouped as follows:
//!
//! - **HTTP/1 Reason Phrase**: [`ReasonPhrase`] — Access non-canonical reason phrases in HTTP/1 responses.
//! - **Informational Responses**: [`on_informational`] — Register callbacks for 1xx HTTP/1 responses on the client.
//! - **Header Case Tracking**: Internal types for tracking the original casing and order of headers as received.
//!
//! Some extensions are only available for HTTP/1 clients or servers.
//!
//! See the documentation on each item for details about its usage and requirements.

#[cfg(all(any(feature = "client", feature = "server"), feature = "http1"))]
use bytes::Bytes;
#[cfg(all(any(feature = "client", feature = "server"), feature = "http1"))]
use http::header::HeaderName;
#[cfg(all(any(feature = "client", feature = "server"), feature = "http1"))]
use http::header::{HeaderMap, IntoHeaderName, ValueIter};
#[cfg(feature = "http1")]
mod h1_reason_phrase;
#[cfg(feature = "http1")]
pub use h1_reason_phrase::ReasonPhrase;

#[cfg(all(feature = "http1", feature = "client"))]
mod informational;
#[cfg(all(feature = "http1", feature = "client"))]
pub use informational::on_informational;
#[cfg(all(feature = "http1", feature = "client"))]
pub(crate) use informational::OnInformational;

/// A map from header names to their original casing as received in an HTTP message.
///
/// If an HTTP/1 response `res` is parsed on a connection whose option
/// [`preserve_header_case`] was set to true and the response included
/// the following headers:
///
/// ```ignore
/// x-Bread: Baguette
/// X-BREAD: Pain
/// x-bread: Ficelle
/// ```
///
/// Then `res.extensions().get::<HeaderCaseMap>()` will return a map with:
///
/// ```ignore
/// HeaderCaseMap({
///     "x-bread": ["x-Bread", "X-BREAD", "x-bread"],
/// })
/// ```
///
/// [`preserve_header_case`]: /client/struct.Client.html#method.preserve_header_case
#[cfg(all(any(feature = "client", feature = "server"), feature = "http1"))]
#[derive(Clone, Debug)]
pub(crate) struct HeaderCaseMap(HeaderMap<Bytes>);

#[cfg(all(any(feature = "client", feature = "server"), feature = "http1"))]
impl HeaderCaseMap {
    /// Returns a view of all spellings associated with that header name,
    /// in the order they were found.
    #[cfg(feature = "client")]
    pub(crate) fn get_all<'hdr>(
        &'hdr self,
        name: &HeaderName,
    ) -> impl Iterator<Item = impl AsRef<[u8]> + 'hdr> + 'hdr {
        self.get_all_internal(name)
    }

    /// Returns a view of all spellings associated with that header name,
    /// in the order they were found.
    #[cfg(any(feature = "client", feature = "server"))]
    pub(crate) fn get_all_internal(&self, name: &HeaderName) -> ValueIter<'_, Bytes> {
        self.0.get_all(name).into_iter()
    }

    #[cfg(any(feature = "client", feature = "server"))]
    pub(crate) fn default() -> Self {
        Self(HeaderMap::default())
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, name: HeaderName, orig: Bytes) {
        self.0.insert(name, orig);
    }

    #[cfg(any(feature = "client", feature = "server"))]
    pub(crate) fn append<N>(&mut self, name: N, orig: Bytes)
    where
        N: IntoHeaderName,
    {
        self.0.append(name, orig);
    }
}
