use crate::fs::asyncify;

use std::io;
use std::path::Path;

/// Returns `Ok(true)` if the path points at an existing entity.
///
/// This function will traverse symbolic links to query information about the
/// destination file. In case of broken symbolic links this will return `Ok(false)`.
///
/// This is the async equivalent of [`std::path::Path::try_exists`][std].
///
/// [std]: fn@std::path::Path::try_exists
///
/// # Examples
///
/// ```no_run
/// use tokio::fs;
///
/// # async fn dox() -> std::io::Result<()> {
/// fs::try_exists("foo.txt").await?;
/// # Ok(())
/// # }
/// ```
pub async fn try_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    let path = path.as_ref();

    try_exists_spawn_blocking(path).await
}

async fn try_exists_spawn_blocking(path: &Path) -> io::Result<bool> {
    let path = path.to_owned();
    // FIXME: When MSRV is 1.81, change this to
    // std::fs::exists() to be consistent with
    // all other tokio::fs operations
    asyncify(move || path.try_exists()).await
}
