//! Source input rules shared by the builder's build script and runtime checks.
//!
//! The compiled implementation basis deliberately remains conservative: it
//! includes the complete foundation, relocated test/include data and legal
//! documentation. The full source verification receipt retains this coverage
//! independently of Cargo's selected compilation units.

use crate::layout;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

pub(crate) type Files = BTreeMap<String, String>;
pub(crate) const MAX_FILE: u64 = 32 * 1024 * 1024;
pub(crate) const MAX_PACKAGE_FILES: usize = 16_384;
pub(crate) const MAX_VERIFICATION_FILES: usize = 131_072;
pub(crate) const VERIFICATION_ROOTS: [&str; 4] = [
    layout::FOUNDATION,
    "docs/dependencies",
    "test",
    "compliance/licenses/upstream",
];

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub(crate) fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn relative(path: &Path) -> io::Result<String> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || !path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(invalid(format!(
            "Uncontrolled relative path: {}",
            path.display()
        )));
    }
    let name = path.to_str().ok_or_else(|| invalid("Path must be UTF-8"))?;
    if name.contains('\\') {
        return Err(invalid(format!(
            "Backslashes are forbidden in source/output paths: {name}"
        )));
    }
    Ok(name.to_owned())
}

pub(crate) fn physical(path: &Path) -> io::Result<()> {
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            return Err(invalid(format!(
                "Symlink input/output is forbidden: {}",
                ancestor.display()
            )));
        }
    }
    Ok(())
}

fn bounded_file(info: &fs::Metadata, path: &Path) -> io::Result<()> {
    if !info.is_file() || info.len() > MAX_FILE {
        return Err(invalid(format!(
            "Expected bounded physical file: {}",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if info.nlink() != 1 {
            return Err(invalid(format!(
                "Hard linked input/output is forbidden: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn same_file(before: &fs::Metadata, after: &fs::Metadata) -> io::Result<bool> {
    let same = before.len() == after.len() && before.modified()? == after.modified()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(same
            && before.dev() == after.dev()
            && before.ino() == after.ino()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec())
    }
    #[cfg(not(unix))]
    Ok(same)
}

/// Bound reads and reject links, replacements or changes while bytes are read.
pub(crate) fn read(path: &Path) -> io::Result<Vec<u8>> {
    physical(path)?;
    let before = fs::symlink_metadata(path)?;
    bounded_file(&before, path)?;
    let mut file = fs::File::open(path)?;
    let opened = file.metadata()?;
    bounded_file(&opened, path)?;
    if !same_file(&before, &opened)? {
        return Err(invalid(format!(
            "Input changed while opening: {}",
            path.display()
        )));
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&mut file).take(MAX_FILE + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    physical(path)?;
    let current = fs::symlink_metadata(path)?;
    bounded_file(&after, path)?;
    bounded_file(&current, path)?;
    if bytes.len() as u64 != before.len()
        || !same_file(&before, &after)?
        || !same_file(&before, &current)?
    {
        return Err(invalid(format!(
            "Input changed while reading: {}",
            path.display()
        )));
    }
    Ok(bytes)
}

fn ignored_name(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(".git" | "__pycache__" | ".pytest_cache")
    )
}

fn generated_web_path(path: &Path, allow_package: bool) -> bool {
    let first = path.components().next().map(|part| part.as_os_str());
    first.is_some_and(|name| {
        layout::WEB_GENERATED_DIRECTORIES
            .iter()
            .any(|excluded| name == *excluded && !(allow_package && *excluded == "pkg"))
            || name.to_str().is_some_and(|name| {
                name.starts_with(".rust-web-stage-") || name.starts_with(".previous-")
            })
    }) || path
        .components()
        .any(|part| ignored_name(part.as_os_str()) || part.as_os_str() == "node_modules")
}

/// All authored Web directories share one policy. Only import resolution may
/// admit logical pkg/ paths, which must be redirected to the current stage.
pub(crate) fn web_relative(path: &Path, allow_package: bool) -> io::Result<String> {
    let name = relative(path)?;
    if generated_web_path(path, allow_package) {
        return Err(invalid(format!(
            "Generated or ignored Web paths cannot be source inputs: {name}"
        )));
    }
    Ok(name)
}

fn append_paths(
    root: &Path,
    paths: &mut Vec<PathBuf>,
    maximum: usize,
    web: Option<&Path>,
) -> io::Result<()> {
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
        Ok(_) => physical(root)?,
    }
    let mut entries = fs::read_dir(root)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let path = entry.path();
        if ignored_name(&name)
            || web.is_none() && name == "target" && root.join("Cargo.toml").is_file()
            || web.is_some_and(|web| {
                generated_web_path(path.strip_prefix(web).expect("Web traversal root"), false)
            })
        {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(invalid(format!(
                "Symlink input/output is forbidden: {}",
                path.display()
            )));
        }
        if kind.is_dir() {
            append_paths(&path, paths, maximum, web)?;
        } else if kind.is_file() {
            paths.push(path);
        } else {
            return Err(invalid(format!(
                "Special object is forbidden: {}",
                path.display()
            )));
        }
        if paths.len() > maximum {
            return Err(invalid("Input/output file count exceeds bound"));
        }
    }
    Ok(())
}

pub(crate) fn walk(root: &Path, maximum: usize) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    append_paths(root, &mut paths, maximum, None)?;
    Ok(paths)
}

pub(crate) fn web_paths(web: &Path, subtree: &Path) -> io::Result<Vec<PathBuf>> {
    physical(web)?;
    if subtree != web {
        web_relative(
            subtree
                .strip_prefix(web)
                .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?,
            false,
        )?;
    }
    let mut paths = Vec::new();
    append_paths(subtree, &mut paths, MAX_PACKAGE_FILES, Some(web))?;
    Ok(paths)
}

pub(crate) fn verification_paths(repository: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for name in VERIFICATION_ROOTS {
        append_paths(
            &repository.join(name),
            &mut paths,
            MAX_VERIFICATION_FILES,
            None,
        )?;
    }
    Ok(paths)
}

/// Preserve the receipt's path-NUL-hex-digest-newline encoding and ordering.
pub(crate) fn digest(files: &Files) -> String {
    let mut digest = Sha256::new();
    for (name, hash) in files {
        digest.update(name.as_bytes());
        digest.update([0]);
        digest.update(hash.as_bytes());
        digest.update(b"\n");
    }
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// A cache belongs to one snapshot only; subsequent snapshots reread all bytes.
#[derive(Default)]
pub(crate) struct SnapshotFiles {
    hashes: BTreeMap<PathBuf, String>,
}

impl SnapshotFiles {
    pub(crate) fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        let bytes = read(path)?;
        self.hashes.insert(path.into(), sha(&bytes));
        Ok(bytes)
    }

    pub(crate) fn hash(&mut self, path: &Path) -> io::Result<String> {
        if let Some(digest) = self.hashes.get(path) {
            return Ok(digest.clone());
        }
        self.read(path)?;
        Ok(self
            .hashes
            .get(path)
            .expect("invariant: reading a source records its digest")
            .clone())
    }

    pub(crate) fn files(&mut self, root: &Path) -> io::Result<Files> {
        self.collect(root, walk(root, MAX_PACKAGE_FILES)?)
    }

    pub(crate) fn collect(&mut self, repository: &Path, paths: Vec<PathBuf>) -> io::Result<Files> {
        paths
            .into_iter()
            .map(|path| {
                let name = relative(
                    path.strip_prefix(repository)
                        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?,
                )?;
                Ok((name, self.hash(&path)?))
            })
            .collect()
    }

    pub(crate) fn verification_inputs(&mut self, repository: &Path) -> io::Result<Files> {
        self.collect(repository, verification_paths(repository)?)
    }
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/source_inputs.rs"]
mod tests;
