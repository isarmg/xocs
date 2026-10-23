//! Compile-time target gate shared by Blog server binaries.
//!
//! A server that depends on this crate cannot be compiled for a different
//! architecture, operating system, or libc environment. Client/client binaries
//! intentionally do not depend on it and keep their own platform contracts.

#[cfg(not(all(
    target_arch = "x86_64",
    target_os = "linux",
    target_env = "gnu",
    target_pointer_width = "64"
)))]
compile_error!(
    "Blog server binaries support only x86_64-unknown-linux-gnu (AMD64); no fallback target is provided"
);

pub const SERVER_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub const SERVER_ARCHITECTURE: &str = "amd64";

/// Assert that release/config metadata has not drifted from the compiled gate.
pub fn require_server_target(value: &str) -> Result<(), UnsupportedServerTarget> {
    if value == SERVER_TARGET_TRIPLE {
        Ok(())
    } else {
        Err(UnsupportedServerTarget {
            actual: value.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedServerTarget {
    actual: String,
}

impl UnsupportedServerTarget {
    pub fn actual(&self) -> &str {
        &self.actual
    }
}

impl std::fmt::Display for UnsupportedServerTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "server target must be {SERVER_TARGET_TRIPLE}, found {:?}",
            self.actual
        )
    }
}

impl std::error::Error for UnsupportedServerTarget {}

#[path = "../../../../test/application/unit/server_target/tests.rs"]
#[cfg(test)]
mod tests;
