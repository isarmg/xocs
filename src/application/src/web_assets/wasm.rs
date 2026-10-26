//! Build-time compressed WASM negotiation around the verified asset responder.
use bytes::Bytes;

/// Serve a precompressed WASM variant when the client accepts gzip.
///
/// `serve` remains responsible for safe asset lookup, representation ETags and
/// GET/HEAD/conditional responses. A missing gzip variant may fall back to
/// identity only when it is acceptable; otherwise an existing resource returns
/// an empty 406. Other files, missing resources and methods pass through intact.
pub fn response(
    path: &str,
    method: &http::Method,
    headers: &http::HeaderMap,
    serve: impl Fn(&str, &http::Method, &http::HeaderMap) -> http::Response<Bytes>,
) -> http::Response<Bytes> {
    if !path.ends_with(".wasm") || (method != http::Method::GET && method != http::Method::HEAD) {
        return serve(path, method, headers);
    }
    let accepted = http::header::accepted_encodings(headers);
    let mut result = if accepted.gzip {
        let compressed = serve(&format!("{path}.gz"), method, headers);
        if compressed.status() == http::StatusCode::OK
            || compressed.status() == http::StatusCode::NOT_MODIFIED
        {
            let mut compressed = compressed;
            compressed.headers_mut().insert(
                http::header::CONTENT_ENCODING,
                http::HeaderValue::from_static("gzip"),
            );
            compressed.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/wasm"),
            );
            compressed
        } else if compressed.status() == http::StatusCode::NOT_FOUND {
            serve(path, method, headers)
        } else {
            compressed
        }
    } else {
        serve(path, method, headers)
    };
    let successful = result.status() == http::StatusCode::OK
        || result.status() == http::StatusCode::NOT_MODIFIED;
    if successful
        && !result
            .headers()
            .contains_key(http::header::CONTENT_ENCODING)
        && !accepted.identity
    {
        return http::Response::builder()
            .status(http::StatusCode::NOT_ACCEPTABLE)
            .header(http::header::CACHE_CONTROL, "no-store")
            .header(http::header::CONTENT_LENGTH, "0")
            .header(http::header::VARY, "Accept-Encoding")
            .body(Bytes::new())
            .expect("constant encoding refusal headers");
    }
    let already_varies = result
        .headers()
        .get_all(http::header::VARY)
        .iter()
        .any(|value| {
            value.to_str().is_ok_and(|v| {
                v.split(',')
                    .any(|s| s.trim() == "*" || s.trim().eq_ignore_ascii_case("accept-encoding"))
            })
        });
    if !already_varies {
        result.headers_mut().append(
            http::header::VARY,
            http::HeaderValue::from_static("Accept-Encoding"),
        );
    }
    result
}

#[path = "../../../../test/application/unit/web_assets/wasm/tests.rs"]
#[cfg(test)]
mod tests;
