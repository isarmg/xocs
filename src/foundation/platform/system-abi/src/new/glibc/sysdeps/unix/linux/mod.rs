//! Source directory: `sysdeps/unix/sysv/linux` (the `sysv` is flattened).
//!
//! <https://github.com/sailfishos-mirror/glibc/tree/master/sysdeps/unix/sysv/linux>

/// Directory: `sysdeps/unix/sysv/linux/bits`
pub(crate) mod bits {

    pub(crate) mod sigaction;

    pub(crate) mod signum_arch;

    pub(crate) mod socket_type;

    pub(crate) mod socket;

    pub(crate) mod statvfs;

    pub(crate) mod types;
}

/// Directory: `net/`
///
/// Source directory: `sysdeps/unix/sysv/linux/net`
pub(crate) mod net {
    pub(crate) mod route;
}
