use super::*;
#[test]
fn gzip_variant_keeps_representation_etag_head_and_304() {
    let serve = |path: &str, method: &http::Method, headers: &http::HeaderMap| {
        let compressed = path.ends_with(".gz");
        let etag = if compressed { "\"gzip\"" } else { "\"raw\"" };
        let unchanged = headers
            .get(http::header::IF_NONE_MATCH)
            .is_some_and(|v| v == etag);
        http::Response::builder()
            .status(if unchanged { 304 } else { 200 })
            .header(http::header::ETAG, etag)
            .header(http::header::CONTENT_TYPE, "application/octet-stream")
            .header(
                http::header::CONTENT_LENGTH,
                if compressed { "4" } else { "3" },
            )
            .body(Bytes::from_static(
                if unchanged || method == http::Method::HEAD {
                    b""
                } else if compressed {
                    b"gzip"
                } else {
                    b"raw"
                },
            ))
            .unwrap()
    };
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::ACCEPT_ENCODING,
        http::HeaderValue::from_static("gzip"),
    );
    let get = response("/assets/app.wasm", &http::Method::GET, &headers, serve);
    assert_eq!(get.body().as_ref(), b"gzip");
    assert_eq!(
        get.headers()[http::header::CONTENT_TYPE],
        "application/wasm"
    );
    assert_eq!(get.headers()[http::header::CONTENT_ENCODING], "gzip");
    assert_eq!(get.headers()[http::header::VARY], "Accept-Encoding");
    let head = response("/assets/app.wasm", &http::Method::HEAD, &headers, serve);
    assert!(head.body().is_empty());
    assert_eq!(head.headers()[http::header::CONTENT_LENGTH], "4");
    headers.insert(
        http::header::IF_NONE_MATCH,
        get.headers()[http::header::ETAG].clone(),
    );
    let cached = response("/assets/app.wasm", &http::Method::GET, &headers, serve);
    assert_eq!(cached.status(), http::StatusCode::NOT_MODIFIED);
    assert!(cached.body().is_empty());
    assert_eq!(cached.headers()[http::header::CONTENT_ENCODING], "gzip");
    headers.remove(http::header::ACCEPT_ENCODING);
    let raw = response("/assets/app.wasm", &http::Method::GET, &headers, serve);
    assert_eq!(raw.status(), http::StatusCode::OK);
    assert_eq!(raw.body().as_ref(), b"raw");
    assert!(!raw.headers().contains_key(http::header::CONTENT_ENCODING));
    assert_ne!(
        raw.headers()[http::header::ETAG],
        get.headers()[http::header::ETAG]
    );
}
#[test]
fn absent_compressed_asset_falls_back_and_errors_stay_errors() {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::ACCEPT_ENCODING,
        http::HeaderValue::from_static("gzip"),
    );
    let serve = |path: &str, _: &http::Method, _: &http::HeaderMap| {
        http::Response::builder()
            .status(if path.ends_with(".gz") { 404 } else { 200 })
            .body(Bytes::from_static(b"raw"))
            .unwrap()
    };
    let raw = response("/app.wasm", &http::Method::GET, &headers, serve);
    assert_eq!(raw.status(), http::StatusCode::OK);
    assert!(!raw.headers().contains_key(http::header::CONTENT_ENCODING));
    let missing = response("/missing.wasm", &http::Method::GET, &headers, |_, _, _| {
        http::Response::builder()
            .status(404)
            .body(Bytes::new())
            .unwrap()
    });
    assert_eq!(missing.status(), http::StatusCode::NOT_FOUND);
    let is_encoded = missing
        .headers()
        .contains_key(http::header::CONTENT_ENCODING);
    assert!(!is_encoded);
}

#[test]
fn compressed_asset_failures_preserve_status_and_vary_instead_of_serving_identity() {
    for encoding in ["gzip", "gzip, identity;q=0"] {
        let mut headers = http::HeaderMap::new();
        headers.insert(http::header::ACCEPT_ENCODING, encoding.parse().unwrap());
        for method in [&http::Method::GET, &http::Method::HEAD] {
            for failure in [
                http::StatusCode::INTERNAL_SERVER_ERROR,
                http::StatusCode::FORBIDDEN,
            ] {
                let result = response("/app.wasm", method, &headers, |path, method, _| {
                    http::Response::builder()
                        .status(if path.ends_with(".gz") {
                            failure
                        } else {
                            http::StatusCode::OK
                        })
                        .header(http::header::CACHE_CONTROL, "no-store")
                        .header(http::header::VARY, "Origin")
                        .body(Bytes::from_static(if method == http::Method::HEAD {
                            b""
                        } else {
                            b"asset read failed"
                        }))
                        .unwrap()
                });
                assert_eq!(result.status(), failure);
                assert_eq!(result.headers()[http::header::CACHE_CONTROL], "no-store");
                assert_eq!(
                    result.body().as_ref(),
                    if method == http::Method::HEAD {
                        &b""[..]
                    } else {
                        &b"asset read failed"[..]
                    },
                );
                let vary: Vec<_> = result
                    .headers()
                    .get_all(http::header::VARY)
                    .iter()
                    .map(|value| value.to_str().unwrap())
                    .collect();
                assert_eq!(vary, ["Origin", "Accept-Encoding"]);
                assert!(
                    !result
                        .headers()
                        .contains_key(http::header::CONTENT_ENCODING)
                );
            }
        }
    }
}
#[test]
fn rejected_identity_is_not_a_compressed_fallback() {
    for encoding in ["gzip;q=0, identity;q=0", "gzip, identity;q=0", "*;q=0"] {
        let mut headers = http::HeaderMap::new();
        headers.insert(http::header::ACCEPT_ENCODING, encoding.parse().unwrap());
        for method in [&http::Method::GET, &http::Method::HEAD] {
            let result = response("/app.wasm", method, &headers, |path, _, _| {
                http::Response::builder()
                    .status(if path.ends_with(".gz") { 404 } else { 200 })
                    .header(http::header::ETAG, "\"raw\"")
                    .body(Bytes::from_static(b"unencoded payload"))
                    .unwrap()
            });
            assert_eq!(
                result.status(),
                http::StatusCode::NOT_ACCEPTABLE,
                "{encoding}"
            );
            assert!(result.body().is_empty());
            assert_eq!(result.headers()[http::header::CONTENT_LENGTH], "0");
            assert_eq!(result.headers()[http::header::VARY], "Accept-Encoding");
            assert!(!result.headers().contains_key(http::header::ETAG));
        }
    }
}
