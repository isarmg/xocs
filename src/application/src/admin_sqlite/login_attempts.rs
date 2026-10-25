//! Bounded registry of physical administrator-database admission scopes.

use crate::admin_core::{AuthenticationState, AuthenticationStateError, LOGIN_ATTEMPTS_SCOPES_MAX};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{
    collections::HashMap,
    fs::File,
    os::unix::fs::MetadataExt,
    sync::{Arc, Mutex, OnceLock, Weak},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Scope {
    File { device: u64, inode: u64 },
    Memory(usize),
}

#[derive(Debug)]
struct Entry {
    state: Arc<AuthenticationState>,
    // Pin memory-scope pointer identity too: an allocator cannot reuse the
    // options address while its unexpired registry entry still exists.
    owner: Weak<SqliteConnectOptions>,
    _memory_identity_pin: Option<Arc<SqliteConnectOptions>>,
    // Keeping the verified main file open prevents an unlinked database's
    // device/inode from being recycled into a different installation scope.
    // O_PATH is essential: closing an ordinary descriptor for the same inode
    // would release this process's SQLite POSIX byte-range locks.
    _database_identity_pin: Option<File>,
}

#[derive(Debug, Default)]
struct Registry {
    entries: HashMap<Scope, Entry>,
}

impl Registry {
    fn acquire(
        &mut self,
        scope: Scope,
        owner: &Arc<SqliteConnectOptions>,
        database_identity_pin: Option<File>,
    ) -> Result<Arc<AuthenticationState>, AuthenticationStateError> {
        if let Some(entry) = self.entries.get_mut(&scope) {
            entry.owner = Arc::downgrade(owner);
            return Ok(entry.state.clone());
        }
        // Never evict a live counter or an unexpired window: recreating a
        // service/pool must not reset the installation's admitted attempts.
        self.entries.retain(|_, entry| {
            entry.owner.strong_count() > usize::from(entry._memory_identity_pin.is_some())
                || Arc::strong_count(&entry.state) != 1
                || !entry.state.can_retire()
        });
        if self.entries.len() == LOGIN_ATTEMPTS_SCOPES_MAX {
            return Err(AuthenticationStateError::Unavailable);
        }
        let state = Arc::new(AuthenticationState::new());
        self.entries.insert(
            scope,
            Entry {
                state: state.clone(),
                owner: Arc::downgrade(owner),
                _memory_identity_pin: matches!(scope, Scope::Memory(_)).then(|| owner.clone()),
                _database_identity_pin: database_identity_pin,
            },
        );
        Ok(state)
    }
}

pub(super) async fn state_for_pool(
    pool: &SqlitePool,
) -> Result<Arc<AuthenticationState>, AuthenticationStateError> {
    let filename: String =
        sqlx::query_scalar("SELECT file FROM pragma_database_list WHERE name='main'")
            .fetch_one(pool)
            .await
            .map_err(|_| AuthenticationStateError::Unavailable)?;
    let owner = pool.connect_options();
    let (scope, database_identity_pin) = if filename.is_empty() {
        (Scope::Memory(Arc::as_ptr(&owner) as usize), None)
    } else {
        let file = rustix::fs::open(
            &filename,
            rustix::fs::OFlags::PATH | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )
        .map(File::from)
        .map_err(|_| AuthenticationStateError::Unavailable)?;
        let metadata = file
            .metadata()
            .map_err(|_| AuthenticationStateError::Unavailable)?;
        let named = std::fs::symlink_metadata(&filename)
            .map_err(|_| AuthenticationStateError::Unavailable)?;
        if !metadata.is_file()
            || !named.is_file()
            || (metadata.dev(), metadata.ino()) != (named.dev(), named.ino())
        {
            return Err(AuthenticationStateError::Unavailable);
        }
        (
            Scope::File {
                device: metadata.dev(),
                inode: metadata.ino(),
            },
            Some(file),
        )
    };
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| Mutex::new(Registry::default()))
        .lock()
        .expect("authentication scope registry lock poisoned")
        .acquire(scope, &owner, database_identity_pin)
}

#[path = "../../../../test/application/unit/admin_sqlite/login_attempts/tests.rs"]
#[cfg(test)]
mod tests;
