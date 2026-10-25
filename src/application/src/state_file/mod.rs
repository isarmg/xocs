//! Linux server private state and runtime instance ownership.
//!
//! This is deliberately narrower than a general path-sandbox library. It owns
//! one already-resolved private directory, direct child state files, stable
//! device/inode checks, and the runtime instance lock.

use rustix::fs::{FlockOperation, OFlags, flock};
use std::{
    ffi::OsStr,
    fs::{self, DirBuilder, File, Metadata, OpenOptions},
    io,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

pub const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
pub const PRIVATE_FILE_MODE: u32 = 0o600;
pub const INSTANCE_LOCK_FILE: &str = ".state-instance.lock";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    device: u64,
    inode: u64,
}

impl FileIdentity {
    pub const fn device(self) -> u64 {
        self.device
    }

    pub const fn inode(self) -> u64 {
        self.inode
    }

    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrivateStateDirectory {
    path: PathBuf,
    identity: FileIdentity,
    owner_uid: u32,
    owner_gid: u32,
    administrative_handle: Option<Arc<File>>,
}

impl PrivateStateDirectory {
    /// Create the directory when missing, then require an absolute, private,
    /// non-symlink directory owned by the effective process user.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        require_absolute(path)?;
        match fs::symlink_metadata(path) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let mut builder = DirBuilder::new();
                builder
                    .mode(PRIVATE_DIRECTORY_MODE)
                    .create(path)
                    .map_err(|source| Error::CreateDirectory {
                        path: path.to_path_buf(),
                        source,
                    })?;
            }
            Err(source) => {
                return Err(Error::InspectPath {
                    path: path.to_path_buf(),
                    source,
                });
            }
        }
        Self::open(path)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        require_absolute(path)?;
        let metadata = symlink_metadata(path)?;
        let owner_uid = rustix::process::geteuid().as_raw();
        require_directory(path, &metadata, owner_uid)?;
        Ok(Self {
            path: path.to_path_buf(),
            identity: FileIdentity::from_metadata(&metadata),
            owner_uid,
            owner_gid: metadata.gid(),
            administrative_handle: None,
        })
    }

    /// Open an existing service-owned private directory under an explicit local administrative identity.
    /// Only its owner or effective uid 0 is authorized. Every path component is
    /// opened without following links; newly created files retain the directory
    /// owner's uid/gid. This never creates, chmods, or chowns an existing object.
    /// Normal runtime access must continue to use [`Self::open`].
    pub fn open_for_administration(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        require_absolute(path)?;
        let directory = open_physical_directory(path)?;
        let metadata = directory.metadata().map_err(|source| Error::InspectPath {
            path: path.to_path_buf(),
            source,
        })?;
        let caller = rustix::process::geteuid().as_raw();
        if caller != 0 && caller != metadata.uid() {
            return Err(Error::UnexpectedOwner {
                path: path.to_path_buf(),
                expected: caller,
                actual: metadata.uid(),
            });
        }
        require_directory(path, &metadata, metadata.uid())?;
        let result = Self {
            path: path.to_path_buf(),
            identity: FileIdentity::from_metadata(&metadata),
            owner_uid: metadata.uid(),
            owner_gid: metadata.gid(),
            administrative_handle: Some(Arc::new(directory)),
        };
        result.verify_identity()?;
        Ok(result)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn identity(&self) -> FileIdentity {
        self.identity
    }

    pub const fn owner_uid(&self) -> u32 {
        self.owner_uid
    }

    pub const fn owner_gid(&self) -> u32 {
        self.owner_gid
    }

    pub fn verify_identity(&self) -> Result<(), Error> {
        if self.administrative_handle.is_some() {
            let caller = rustix::process::geteuid().as_raw();
            if caller != 0 && caller != self.owner_uid {
                return Err(Error::UnexpectedOwner {
                    path: self.path.clone(),
                    expected: caller,
                    actual: self.owner_uid,
                });
            }
        }
        let metadata = if self.administrative_handle.is_some() {
            open_physical_directory(&self.path)?
                .metadata()
                .map_err(|source| Error::InspectPath {
                    path: self.path.clone(),
                    source,
                })?
        } else {
            symlink_metadata(&self.path)?
        };
        require_directory(&self.path, &metadata, self.owner_uid)?;
        if self.administrative_handle.is_some()
            && metadata.mode() & 0o7777 != PRIVATE_DIRECTORY_MODE
        {
            return Err(Error::UnexpectedMode {
                path: self.path.clone(),
                expected: PRIVATE_DIRECTORY_MODE,
                actual: metadata.mode() & 0o7777,
            });
        }
        if self.administrative_handle.is_some() && metadata.gid() != self.owner_gid {
            return Err(Error::FileIdentityChanged {
                path: self.path.clone(),
                expected: self.identity,
                actual: FileIdentity::from_metadata(&metadata),
            });
        }
        require_identity(
            &self.path,
            self.identity,
            FileIdentity::from_metadata(&metadata),
        )
    }

    pub fn open_existing(&self, name: impl AsRef<OsStr>) -> Result<SecureStateFile, Error> {
        self.open_file(name.as_ref(), false)
    }

    pub fn create_file(&self, name: impl AsRef<OsStr>) -> Result<SecureStateFile, Error> {
        self.open_file(name.as_ref(), true)
    }

    pub fn try_instance_lock(&self) -> Result<InstanceLock, Error> {
        self.verify_identity()?;
        let instance = self.create_file(INSTANCE_LOCK_FILE)?;
        let instance = StateLock::acquire(
            instance,
            FlockOperation::NonBlockingLockExclusive,
            LockKind::Instance,
        )?;
        instance.verify_identity()?;
        self.verify_identity()?;
        Ok(InstanceLock {
            directory: self.clone(),
            _instance: instance,
        })
    }

    fn open_file(&self, name: &OsStr, create: bool) -> Result<SecureStateFile, Error> {
        self.verify_identity()?;
        let relative = Path::new(name);
        require_direct_child(relative)?;
        let path = self.path.join(relative);
        let file = if let Some(directory) = &self.administrative_handle {
            administrative_file(directory, name, create, self.owner_uid, self.owner_gid).map_err(
                |source| Error::OpenFile {
                    path: path.clone(),
                    source,
                },
            )?
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(create)
                .mode(PRIVATE_FILE_MODE)
                .custom_flags(OFlags::NOFOLLOW.bits() as i32)
                .open(&path)
                .map_err(|source| Error::OpenFile {
                    path: path.clone(),
                    source,
                })?
        };
        let metadata = file.metadata().map_err(|source| Error::InspectPath {
            path: path.clone(),
            source,
        })?;
        require_file(&path, &metadata, self.owner_uid)?;
        let identity = FileIdentity::from_metadata(&metadata);
        let path_metadata = symlink_metadata(&path)?;
        require_file(&path, &path_metadata, self.owner_uid)?;
        require_identity(&path, identity, FileIdentity::from_metadata(&path_metadata))?;
        self.verify_identity()?;
        Ok(SecureStateFile {
            file,
            path,
            identity,
            owner_uid: self.owner_uid,
        })
    }
}

#[derive(Debug)]
pub struct SecureStateFile {
    file: File,
    path: PathBuf,
    identity: FileIdentity,
    owner_uid: u32,
}

impl SecureStateFile {
    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn identity(&self) -> FileIdentity {
        self.identity
    }

    /// Re-check both the open descriptor and its path. This detects replacement
    /// between the initial check and a database driver's independent open.
    pub fn verify_identity(&self) -> Result<(), Error> {
        let descriptor = self.file.metadata().map_err(|source| Error::InspectPath {
            path: self.path.clone(),
            source,
        })?;
        require_file(&self.path, &descriptor, self.owner_uid)?;
        require_identity(
            &self.path,
            self.identity,
            FileIdentity::from_metadata(&descriptor),
        )?;
        let path_metadata = symlink_metadata(&self.path)?;
        require_file(&self.path, &path_metadata, self.owner_uid)?;
        require_identity(
            &self.path,
            self.identity,
            FileIdentity::from_metadata(&path_metadata),
        )
    }
}

#[derive(Debug)]
struct StateLock {
    file: SecureStateFile,
    kind: LockKind,
    held: bool,
}

impl StateLock {
    fn acquire(
        file: SecureStateFile,
        operation: FlockOperation,
        kind: LockKind,
    ) -> Result<Self, Error> {
        try_flock(file.file(), operation, kind)?;
        // Own the successful lock immediately, including all subsequent
        // identity-check error paths. A fork/dup alias can outlive our file.
        Ok(Self {
            file,
            kind,
            held: true,
        })
    }

    fn unlock(&mut self) -> Result<(), Error> {
        if self.held {
            try_flock(self.file.file(), FlockOperation::Unlock, self.kind)?;
            // An alias may acquire another lock on this description after
            // handoff. Drop must never unlock that newly acquired ownership.
            self.held = false;
        }
        Ok(())
    }
}

impl std::ops::Deref for StateLock {
    type Target = SecureStateFile;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        if self.held {
            // Release this open file description before closing it, even
            // when inherited aliases remain. Cleanup only touches the held
            // descriptor; it never reopens, unlinks or repairs a state path.
            if let Err(error) = self.file.file().unlock() {
                tracing::error!(%error, "state lock finalizer could not release its descriptor");
            }
        }
    }
}

#[derive(Debug)]
pub struct InstanceLock {
    directory: PrivateStateDirectory,
    _instance: StateLock,
}

impl InstanceLock {
    /// Call before handing a separately opened database to a storage driver.
    pub fn verify_identity(&self) -> Result<(), Error> {
        self.directory.verify_identity()?;
        self._instance.verify_identity()
    }

    /// Explicitly hand off write ownership after all business resources close.
    /// Unlocking the open file description also covers descriptors inherited
    /// briefly by concurrently spawned processes before their exec boundary.
    pub fn release(mut self) -> Result<(), Error> {
        self.verify_identity()?;
        self._instance.unlock()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockKind {
    Instance,
}

fn try_flock(file: &File, operation: FlockOperation, kind: LockKind) -> Result<(), Error> {
    flock(file, operation).map_err(|source| Error::LockUnavailable {
        kind,
        source: source.into(),
    })
}

fn open_physical_directory(path: &Path) -> Result<File, Error> {
    use rustix::fs::{Mode, open, openat};
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let fail = |source: rustix::io::Errno| Error::InspectPath {
        path: path.to_path_buf(),
        source: source.into(),
    };
    let mut directory = open("/", flags, Mode::empty()).map_err(fail)?;
    for component in path.components() {
        match component {
            Component::RootDir => (),
            Component::Normal(name) => {
                directory = openat(&directory, name, flags, Mode::empty()).map_err(fail)?
            }
            _ => {
                return Err(Error::InvalidChildName {
                    name: path.to_path_buf(),
                });
            }
        }
    }
    Ok(File::from(directory))
}

fn administrative_file(
    directory: &File,
    name: &OsStr,
    create: bool,
    uid: u32,
    gid: u32,
) -> io::Result<File> {
    use rustix::fs::{Mode, fchown, openat};
    let flags = OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    if create {
        match openat(
            directory,
            name,
            flags | OFlags::CREATE | OFlags::EXCL,
            Mode::from_raw_mode(PRIVATE_FILE_MODE),
        ) {
            Ok(fd) => {
                let file = File::from(fd);
                // This descriptor was created exclusively, so an existing
                // service file can never be repaired or taken over here.
                fchown(
                    &file,
                    Some(rustix::process::Uid::from_raw(uid)),
                    Some(rustix::process::Gid::from_raw(gid)),
                )
                .map_err(io::Error::from)?;
                file.sync_all()?;
                directory.sync_all()?;
                return Ok(file);
            }
            Err(rustix::io::Errno::EXIST) => (),
            Err(error) => return Err(error.into()),
        }
    }
    openat(directory, name, flags, Mode::empty())
        .map(File::from)
        .map_err(io::Error::from)
}

fn require_absolute(path: &Path) -> Result<(), Error> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(Error::StateDirectoryNotAbsolute {
            path: path.to_path_buf(),
        })
    }
}

fn require_direct_child(path: &Path) -> Result<(), Error> {
    let mut components = path.components();
    if matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none() {
        Ok(())
    } else {
        Err(Error::InvalidChildName {
            name: path.to_path_buf(),
        })
    }
}

fn symlink_metadata(path: &Path) -> Result<Metadata, Error> {
    fs::symlink_metadata(path).map_err(|source| Error::InspectPath {
        path: path.to_path_buf(),
        source,
    })
}

fn require_directory(path: &Path, metadata: &Metadata, owner_uid: u32) -> Result<(), Error> {
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::NotDirectory {
            path: path.to_path_buf(),
        });
    }
    require_owner_and_mode(path, metadata, owner_uid, PRIVATE_DIRECTORY_MODE)
}

fn require_file(path: &Path, metadata: &Metadata, owner_uid: u32) -> Result<(), Error> {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::NotRegularFile {
            path: path.to_path_buf(),
        });
    }
    if metadata.nlink() != 1 {
        return Err(Error::UnexpectedHardLinkCount {
            path: path.to_path_buf(),
            actual: metadata.nlink(),
        });
    }
    require_owner_and_mode(path, metadata, owner_uid, PRIVATE_FILE_MODE)
}

fn require_owner_and_mode(
    path: &Path,
    metadata: &Metadata,
    owner_uid: u32,
    expected_mode: u32,
) -> Result<(), Error> {
    if metadata.uid() != owner_uid {
        return Err(Error::UnexpectedOwner {
            path: path.to_path_buf(),
            expected: owner_uid,
            actual: metadata.uid(),
        });
    }
    let actual = metadata.mode() & 0o777;
    if actual != expected_mode {
        return Err(Error::UnexpectedMode {
            path: path.to_path_buf(),
            expected: expected_mode,
            actual,
        });
    }
    Ok(())
}

fn require_identity(
    path: &Path,
    expected: FileIdentity,
    actual: FileIdentity,
) -> Result<(), Error> {
    if expected == actual {
        Ok(())
    } else {
        Err(Error::FileIdentityChanged {
            path: path.to_path_buf(),
            expected,
            actual,
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("state directory must be absolute: {path}", path = .path.display())]
    StateDirectoryNotAbsolute { path: PathBuf },
    #[error("state file name must be one normal direct child: {name}", name = .name.display())]
    InvalidChildName { name: PathBuf },
    #[error("cannot inspect state path {path}: {source}", path = .path.display())]
    InspectPath { path: PathBuf, source: io::Error },
    #[error("cannot create private state directory {path}: {source}", path = .path.display())]
    CreateDirectory { path: PathBuf, source: io::Error },
    #[error("cannot securely open state file {path}: {source}", path = .path.display())]
    OpenFile { path: PathBuf, source: io::Error },
    #[error("state path is not a real directory: {path}", path = .path.display())]
    NotDirectory { path: PathBuf },
    #[error("state path is not one regular no-follow file: {path}", path = .path.display())]
    NotRegularFile { path: PathBuf },
    #[error("state path {path} has owner {actual}, expected {expected}", path = .path.display())]
    UnexpectedOwner {
        path: PathBuf,
        expected: u32,
        actual: u32,
    },
    #[error("state path {path} has mode {actual:o}, expected {expected:o}", path = .path.display())]
    UnexpectedMode {
        path: PathBuf,
        expected: u32,
        actual: u32,
    },
    #[error("state file {path} has {actual} hard links; exactly one is required", path = .path.display())]
    UnexpectedHardLinkCount { path: PathBuf, actual: u64 },
    #[error("state file identity changed for {path}: expected {expected:?}, found {actual:?}", path = .path.display())]
    FileIdentityChanged {
        path: PathBuf,
        expected: FileIdentity,
        actual: FileIdentity,
    },
    #[error("{kind:?} lock is unavailable: {source}")]
    LockUnavailable { kind: LockKind, source: io::Error },
}

#[path = "../../../../test/application/unit/state_file/tests.rs"]
#[cfg(test)]
mod tests;
