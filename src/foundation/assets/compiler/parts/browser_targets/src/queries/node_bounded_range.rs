use super::{Distrib, QueryResult, node_version::NodeVersion};
use crate::browser_targets::data::node::node_versions;

pub(super) fn node_bounded_range(from: &str, to: &str) -> QueryResult {
    let from = NodeVersion::parse(from)?;
    let to = NodeVersion::parse(to)?;
    let distribs = node_versions()
        .iter()
        .filter(|(version, _)| from.compare_to(*version).is_le() && to.compare_to(*version).is_ge())
        .map(|(_, text)| Distrib::new("node", text.as_ref()))
        .collect();
    Ok(distribs)
}
