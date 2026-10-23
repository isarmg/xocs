//! Native POSIX threading definitions for the Linux amd64 GNU ABI.
#[path = "../x86/nptl/bits/struct_mutex.rs"]
pub(crate) mod x86_struct_mutex;
pub(crate) mod bits {
    pub(crate) use super::x86_struct_mutex as struct_mutex;
}
pub(crate) mod pthread;
