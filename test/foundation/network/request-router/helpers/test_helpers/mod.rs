#![allow(clippy::disallowed_names)]

use crate::{extract::Request, response::Response, serve};

mod test_client;
pub use self::test_client::*;

#[cfg(test)]
#[path = "../../legacy/modules/src/test_helpers/tracing_helpers.rs"]
pub(crate) mod tracing_helpers;

#[cfg(test)]
#[path = "../../legacy/modules/src/test_helpers/counting_cloneable_state.rs"]
pub(crate) mod counting_cloneable_state;

#[cfg(test)]
include!("../../unit/src/test_helpers/mod/460.rs");
#[cfg(test)]
include!("../../unit/src/test_helpers/mod/513.rs");

#[allow(dead_code)]
pub(crate) struct NotSendSync(*const ());
