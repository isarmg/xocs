//! Build-bound Web resources and shared HTTP semantics, independent of the server framework.
//!
//! Use [`build::generate`] in `build.rs` and include its generated source at runtime.
//! [`DirectoryAssets`] is an explicitly selected development provider. It never replaces
//! embedded resources automatically, and rereads each requested file for hot reload.

use bytes::Bytes;
use http::{HeaderMap, Method, Response, StatusCode, header};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

pub mod build;
mod error;
pub mod wasm;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

/// Inventory format identity. Its canonical compact JSON bytes are hashed verbatim.
pub const MANIFEST_FORMAT: &str = "blog-browser-assets-v1";
const MAX_ASSET_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_FILES: usize = 16_384;

/// A generated resource; `sha256` binds the exact bytes, not its filename.
#[derive(Clone, Copy, Debug)]
pub struct EmbeddedAsset {
    pub path: &'static str,
    pub content_type: &'static str,
    pub bytes: &'static [u8],
    pub sha256: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetManifest {
    pub format: String,
    pub files: Vec<AssetRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRecord {
    pub path: String,
    pub content_type: String,
    pub size: u64,
    pub sha256: String,
}

/// A manifest whose paths, sizes, serialization and digest have been verified.
#[derive(Debug)]
pub struct VerifiedManifest(AssetManifest);

impl VerifiedManifest {
    pub fn files(&self) -> &[AssetRecord] {
        &self.0.files
    }
}

/// Verify inventory format, canonical ordering/serialization and its bound digest.
pub fn verify_manifest(manifest: &str, digest: &str) -> Result<VerifiedManifest> {
    if manifest.len() > 8 * 1024 * 1024 {
        return Err(Error::ManifestBudget);
    }
    let parsed: AssetManifest = serde_json::from_str(manifest)?;
    if parsed.format != MANIFEST_FORMAT {
        return Err(Error::ManifestFormat);
    }
    if parsed.files.is_empty() || parsed.files.len() > MAX_FILES {
        return Err(Error::InventorySize);
    }
    let mut previous: Option<&str> = None;
    let mut total = 0_u64;
    for file in &parsed.files {
        AssetPath::parse(&file.path)?;
        if previous.is_some_and(|path| path >= file.path.as_str()) {
            return Err(Error::InventoryOrder);
        }
        previous = Some(&file.path);
        if file.content_type != content_type(&file.path) {
            return Err(Error::ContentType);
        }
        if !valid_digest(&file.sha256) {
            return Err(Error::AssetDigest);
        }
        if file.size > MAX_ASSET_BYTES {
            return Err(Error::AssetBudget);
        }
        total += file.size;
        if total > MAX_TOTAL_BYTES {
            return Err(Error::TotalBudget);
        }
    }
    if serde_json::to_string(&parsed)? != manifest {
        return Err(Error::ManifestCanonical);
    }
    if !valid_digest(digest) || sha256(manifest.as_bytes()) != digest {
        return Err(Error::ManifestDigest);
    }
    Ok(VerifiedManifest(parsed))
}

/// Verify a generated registry against the manifest and every embedded byte.
/// Intended for build acceptance and release validation, not every HTTP request.
pub fn verify_embedded(assets: &[EmbeddedAsset], manifest: &str, digest: &str) -> Result<()> {
    let parsed = verify_manifest(manifest, digest)?;
    if parsed.files().len() != assets.len() {
        return Err(Error::EmbeddedLength);
    }
    for (record, asset) in parsed.files().iter().zip(assets) {
        if record.path != asset.path || record.content_type != asset.content_type {
            return Err(Error::EmbeddedMetadata);
        }
        if record.size != asset.bytes.len() as u64
            || record.sha256 != asset.sha256
            || record.sha256 != sha256(asset.bytes)
        {
            return Err(Error::EmbeddedBytes);
        }
    }
    Ok(())
}

/// Serve exact embedded resources. Empty path and `/` select `index.html`.
/// Unknown paths are 404, unsupported methods are 405; no SPA fallback is implicit.
/// HTML is never stored. Other resources revalidate through a strong SHA-256 ETag.
/// The generated registry must be strictly path-sorted, as checked by verify_embedded.
pub fn response(
    assets: &'static [EmbeddedAsset],
    path: &str,
    method: &Method,
    headers: &HeaderMap,
) -> Response<Bytes> {
    if method != Method::GET && method != Method::HEAD {
        return error_response(StatusCode::METHOD_NOT_ALLOWED, method);
    }
    let Some(path) = request_path(path) else {
        return error_response(StatusCode::NOT_FOUND, method);
    };
    let Ok(index) = assets.binary_search_by(|asset| asset.path.cmp(path.0)) else {
        return error_response(StatusCode::NOT_FOUND, method);
    };
    let asset = &assets[index];
    asset_response(
        Bytes::from_static(asset.bytes),
        asset.content_type,
        asset.sha256,
        method,
        headers,
    )
}

/// Descriptor-bound development root, with no-follow reads on Unix.
/// File changes are visible on the next request. Symlinks, hardlinks, special files,
/// unsafe paths, and oversized reads are rejected instead of being served.
#[derive(Clone, Debug)]
pub struct DirectoryAssets {
    root: PathBuf,
    directory: Arc<File>,
}

impl DirectoryAssets {
    /// Open an absolute development directory. No production mode is selected here.
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let result = Self::open(root)?;
        // Fail configuration early for unsafe initial trees; later reads still
        // validate descriptors because the development tree can change.
        build::collect_paths(root, false)?;
        Ok(result)
    }

    fn open(root: &Path) -> Result<Self> {
        if !root.is_absolute() {
            return Err(Error::RelativeDirectory);
        }
        if !root
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Normal(_)))
        {
            return Err(Error::UnsafeDirectory);
        }
        Ok(Self {
            root: root.to_path_buf(),
            directory: Arc::new(open_directory(root)?),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn response(&self, path: &str, method: &Method, headers: &HeaderMap) -> Response<Bytes> {
        if method != Method::GET && method != Method::HEAD {
            return error_response(StatusCode::METHOD_NOT_ALLOWED, method);
        }
        let Some(path) = request_path(path) else {
            return error_response(StatusCode::NOT_FOUND, method);
        };
        match self.read(path) {
            Ok(bytes) => {
                let digest = sha256(&bytes);
                asset_response(
                    Bytes::from(bytes),
                    content_type(path.0),
                    &digest,
                    method,
                    headers,
                )
            }
            Err(Error::UnsafeFile | Error::AssetBudget) => {
                error_response(StatusCode::NOT_FOUND, method)
            }
            Err(Error::Descriptor(
                rustix::io::Errno::NOENT | rustix::io::Errno::NOTDIR | rustix::io::Errno::LOOP,
            )) => error_response(StatusCode::NOT_FOUND, method),
            Err(error) => {
                tracing::error!(error = ?error, "development asset read failed");
                error_response(StatusCode::INTERNAL_SERVER_ERROR, method)
            }
        }
    }

    fn read(&self, path: AssetPath<'_>) -> Result<Vec<u8>> {
        use rustix::fs::{FileType, Mode, OFlags, fstat, openat};
        let mut parent = self.directory.try_clone()?;
        let mut components = Path::new(path.0).components().peekable();
        while let Some(Component::Normal(name)) = components.next() {
            if components.peek().is_some() {
                parent = File::from(openat(&parent, name, directory_flags(), Mode::empty())?);
            } else {
                let file = File::from(openat(
                    &parent,
                    name,
                    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                    Mode::empty(),
                )?);
                let metadata = fstat(&file)?;
                if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile
                    || metadata.st_nlink != 1
                {
                    return Err(Error::UnsafeFile);
                }
                return read_bounded(file);
            }
        }
        unreachable!("invariant: a validated asset path contains a normal component")
    }
}

fn read_bounded(file: File) -> Result<Vec<u8>> {
    if file.metadata()?.len() > MAX_ASSET_BYTES {
        return Err(Error::AssetBudget);
    }
    let mut bytes = Vec::new();
    file.take(MAX_ASSET_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_ASSET_BYTES {
        return Err(Error::AssetBudget);
    }
    Ok(bytes)
}

fn directory_flags() -> rustix::fs::OFlags {
    use rustix::fs::OFlags;
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}

fn open_directory(path: &Path) -> Result<File> {
    use rustix::fs::{Mode, open, openat};
    let mut fd = open("/", directory_flags(), Mode::empty())?;
    for component in path.components() {
        match component {
            Component::RootDir => (),
            Component::Normal(name) => fd = openat(&fd, name, directory_flags(), Mode::empty())?,
            Component::Prefix(_) | Component::CurDir | Component::ParentDir => {
                unreachable!("invariant: the directory path has already been validated")
            }
        }
    }
    Ok(File::from(fd))
}

#[derive(Clone, Copy)]
struct AssetPath<'a>(&'a str);

impl<'a> AssetPath<'a> {
    fn parse(path: &'a str) -> Result<Self> {
        if safe_relative(path) {
            Ok(Self(path))
        } else {
            Err(Error::UnsafePath)
        }
    }
}

fn request_path(path: &str) -> Option<AssetPath<'_>> {
    if path.is_empty() || path == "/" {
        Some(AssetPath("index.html"))
    } else {
        let path = path.strip_prefix('/').unwrap_or(path);
        safe_relative(path).then_some(AssetPath(path))
    }
}

pub(crate) fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.len() <= 255
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Explicit MIME mappings; unknown outputs remain binary and cannot be sniffed.
pub fn content_type(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .expect("invariant: split always yields at least one item")
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "txt" | "md" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "mp4" => "video/mp4",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "wasm" => "application/wasm",
        "webmanifest" => "application/manifest+json",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}

fn base_response(status: StatusCode, content_type: &str, size: usize) -> Response<Bytes> {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::REFERRER_POLICY, "same-origin");
    if status != StatusCode::NOT_MODIFIED {
        builder = builder.header(header::CONTENT_LENGTH, size.to_string());
    }
    builder
        .body(Bytes::new())
        .expect("constant HTTP headers are valid")
}

fn asset_response(
    bytes: Bytes,
    content_type: &str,
    digest: &str,
    method: &Method,
    headers: &HeaderMap,
) -> Response<Bytes> {
    let html = content_type.starts_with("text/html");
    let etag = format!("\"{digest}\"");
    // If-None-Match uses weak comparison for GET/HEAD, including lists and wildcard.
    let unchanged = !html
        && headers.get_all(header::IF_NONE_MATCH).iter().any(|value| {
            value.to_str().is_ok_and(|value| {
                value.split(',').any(|candidate| {
                    let candidate = candidate.trim();
                    candidate == "*" || candidate.strip_prefix("W/").unwrap_or(candidate) == etag
                })
            })
        });
    let mut result = base_response(
        if unchanged {
            StatusCode::NOT_MODIFIED
        } else {
            StatusCode::OK
        },
        content_type,
        bytes.len(),
    );
    result.headers_mut().insert(
        header::CACHE_CONTROL,
        if html {
            "no-store, max-age=0"
        } else {
            "public, no-cache"
        }
        .parse()
        .expect("constant cache header"),
    );
    if !html {
        result
            .headers_mut()
            .insert(header::ETAG, etag.parse().expect("hexadecimal ETag"));
    }
    if method != Method::HEAD && !unchanged {
        *result.body_mut() = bytes;
    }
    result
}

fn error_response(status: StatusCode, method: &Method) -> Response<Bytes> {
    let message: &'static [u8] = if status == StatusCode::METHOD_NOT_ALLOWED {
        b"Method not allowed\n"
    } else if status == StatusCode::NOT_FOUND {
        b"Not found\n"
    } else {
        b"Internal server error\n"
    };
    let mut result = base_response(status, "text/plain; charset=utf-8", message.len());
    result.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store".parse().expect("constant cache header"),
    );
    if status == StatusCode::METHOD_NOT_ALLOWED {
        result.headers_mut().insert(
            header::ALLOW,
            "GET, HEAD".parse().expect("constant allow header"),
        );
    }
    if method != Method::HEAD {
        *result.body_mut() = Bytes::from_static(message);
    }
    result
}

#[cfg(test)]
#[path = "../../../../test/application/unit/web_assets/tests.rs"]
mod tests;
