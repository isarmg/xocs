use super::{Distrib, QueryResult, node_version::NodeVersion};
use crate::browser_targets::{data::node::node_versions, parser::Comparator};

pub(super) fn node_unbounded_range(comparator: Comparator, version: &str) -> QueryResult {
    let version = NodeVersion::parse(version)?.padded();
    let distribs = node_versions()
        .iter()
        .filter(|(v, _)| comparator.compare(*v, version))
        .map(|(_, text)| Distrib::new("node", text.as_ref()))
        .collect();
    Ok(distribs)
}
