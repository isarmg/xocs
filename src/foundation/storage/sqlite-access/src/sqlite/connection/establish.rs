use crate::sqlite::connection::handle::ConnectionHandle;
use crate::sqlite::connection::LogSettings;
use crate::sqlite::connection::{ConnectionState, Statements};
use crate::sqlite::error::Error;
use crate::sqlite::SqliteConnectOptions;
use libsqlite3_sys::{
    sqlite3_busy_timeout, SQLITE_OPEN_CREATE, SQLITE_OPEN_FULLMUTEX, SQLITE_OPEN_MEMORY,
    SQLITE_OPEN_NOMUTEX, SQLITE_OPEN_PRIVATECACHE, SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE,
    SQLITE_OPEN_SHAREDCACHE, SQLITE_OPEN_URI,
};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use url_encoding::percent::NON_ALPHANUMERIC;

// This was originally `AtomicU64` but that's not supported on MIPS (or PowerPC):
// https://github.com/launchbadge/sqlx/issues/2859
// https://doc.rust-lang.org/stable/std/sync/atomic/index.html#portability
static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

pub struct EstablishParams {
    filename: CString,
    open_flags: i32,
    busy_timeout: Duration,
    statement_cache_capacity: usize,
    log_settings: LogSettings,
    pub(crate) thread_name: String,
    pub(crate) command_channel_size: usize,
}

impl EstablishParams {
    pub fn from_options(options: &SqliteConnectOptions) -> Result<Self, Error> {
        let mut filename = options
            .filename
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "filename passed to SQLite must be valid UTF-8",
                )
            })?
            .to_owned();

        // Set common flags we expect to have in sqlite
        let mut flags = SQLITE_OPEN_URI;

        // By default, we connect to an in-memory database.
        // [SQLITE_OPEN_NOMUTEX] will instruct [sqlite3_open_v2] to return an error if it
        // cannot satisfy our wish for a thread-safe, lock-free connection object

        flags |= if options.serialized {
            SQLITE_OPEN_FULLMUTEX
        } else {
            SQLITE_OPEN_NOMUTEX
        };

        flags |= if options.read_only {
            SQLITE_OPEN_READONLY
        } else if options.create_if_missing {
            SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE
        } else {
            SQLITE_OPEN_READWRITE
        };

        if options.in_memory {
            flags |= SQLITE_OPEN_MEMORY;
        }

        flags |= if options.shared_cache {
            SQLITE_OPEN_SHAREDCACHE
        } else {
            SQLITE_OPEN_PRIVATECACHE
        };

        let mut query_params = BTreeMap::new();

        if options.immutable {
            query_params.insert("immutable", "true");
        }

        if let Some(vfs) = options.vfs.as_deref() {
            query_params.insert("vfs", vfs);
        }

        if !query_params.is_empty() {
            filename = format!(
                "file:{}?",
                url_encoding::percent::percent_encode(filename.as_bytes(), NON_ALPHANUMERIC),
            );

            // Suffix serializer automatically handles `&` separators for us.
            let filename_len = filename.len();
            filename = url_encoding::form::Serializer::for_suffix(filename, filename_len)
                .extend_pairs(query_params)
                .finish();
        }

        let filename = CString::new(filename).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "filename passed to SQLite must not contain nul bytes",
            )
        })?;

        let thread_id = THREAD_ID.fetch_add(1, Ordering::AcqRel);

        Ok(Self {
            filename,
            open_flags: flags,
            busy_timeout: options.busy_timeout,
            statement_cache_capacity: options.statement_cache_capacity,
            log_settings: options.log_settings.clone(),
            thread_name: (options.thread_name)(thread_id as u64),
            command_channel_size: options.command_channel_size,
        })
    }

    pub(crate) fn establish(&self) -> Result<ConnectionState, Error> {
        let mut handle = ConnectionHandle::open(&self.filename, self.open_flags)?;

        // Configure a busy timeout
        // This causes SQLite to automatically sleep in increasing intervals until the time
        // when there is something locked during [sqlite3_step].
        //
        // We also need to convert the u128 value to i32, checking we're not overflowing.
        let ms = i32::try_from(self.busy_timeout.as_millis())
            .expect("Given busy timeout value is too big.");

        handle.call_with_result(|db| unsafe { sqlite3_busy_timeout(db, ms) })?;

        Ok(ConnectionState {
            handle,
            statements: Statements::new(self.statement_cache_capacity),
            log_settings: self.log_settings.clone(),
            progress_handler_callback: None,
            update_hook_callback: None,
            commit_hook_callback: None,
            rollback_hook_callback: None,
        })
    }
}
