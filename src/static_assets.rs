use axum::{extract::Request, response::Response};

#[path = "xcss_web_assets.rs"]
mod xcss_web_assets;

include!(concat!(env!("OUT_DIR"), "/xcss-web-assets.rs"));

pub fn verify() -> anyhow::Result<()> {
    xcss::web_assets::verify_embedded(ASSETS, MANIFEST, DIGEST)
}

pub async fn serve(request: Request) -> Response {
    xcss_web_assets::response(
        request.uri().path(),
        request.method(),
        request.headers(),
        |path, method, headers| xcss::web_assets::response(ASSETS, path, method, headers),
    )
    .map(axum::body::Body::from)
}

pub async fn entry(request: Request) -> Response {
    xcss::web_assets::response(ASSETS, "/index.html", request.method(), request.headers())
        .map(axum::body::Body::from)
}
