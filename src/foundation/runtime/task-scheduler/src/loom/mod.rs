//! This module abstracts over `loom` and `std::sync` depending on whether we
//! are running tests or not.

#![allow(unused)]

#[cfg(not(all(test, loom)))]
mod std;
#[cfg(not(all(test, loom)))]
pub(crate) use self::std::*;

#[path = "../../../../../../test/foundation/runtime/task-scheduler/unit/src/loom/mocked.rs"]
#[cfg(all(test, loom))]
mod mocked;
#[cfg(test)]
include!("../../../../../../test/foundation/runtime/task-scheduler/unit/src/loom/mod/261.rs");
