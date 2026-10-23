use super::{Distrib, QueryResult};
use crate::browser_targets::error::Error;

pub(super) fn current_node() -> QueryResult {
    use std::{process::Command, sync::OnceLock};

    static CURRENT_NODE: OnceLock<QueryResult> = OnceLock::new();

    CURRENT_NODE
        .get_or_init(|| {
            let output = Command::new("node")
                .arg("-v")
                .output()
                .map_err(|_| Error::UnsupportedCurrentNode)?;
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_start_matches('v')
                .to_owned();

            Ok(vec![Distrib::new("node", version)])
        })
        .clone()
}
