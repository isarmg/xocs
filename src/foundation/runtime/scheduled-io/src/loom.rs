//! This module abstracts over `loom` and `std::sync` types depending on whether we
//! are running loom tests or not.

pub(crate) mod sync {
    #[cfg(test)]
include!("../../../../../test/foundation/runtime/scheduled-io/unit/src/loom/sync/146.rs");
    #[cfg(not(all(test, loom)))]
    pub(crate) use std::sync::{Arc, Mutex, MutexGuard};
}
