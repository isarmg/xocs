use super::QueryResult;
use crate::browser_targets::{opts::Opts, resolve};

pub(super) fn defaults(opts: &Opts) -> QueryResult {
    resolve(&["> 0.5%", "last 2 versions", "Firefox ESR", "not dead"], opts)
}
