use super::{Distrib, QueryResult, node_version::NodeVersion};
use crate::browser_targets::{data::node::node_versions, error::Error, opts::Opts};

pub(super) fn node_accurate(version_str: &str, opts: &Opts) -> QueryResult {
    // Reject leading zeros (e.g. "08"), like the JavaScript implementation.
    if version_str
        .split('.')
        .any(|n| n != "0" && n.starts_with('0'))
    {
        return Err(Error::UnknownNodejsVersion(version_str.to_string()));
    }

    let version = NodeVersion::parse(version_str)?;

    let distribs = node_versions()
        .iter()
        .rev()
        .find(|(candidate, _)| version.matches(*candidate))
        .map(|(_, text)| vec![Distrib::new("node", text.as_ref())]);
    if opts.ignore_unknown_versions {
        Ok(distribs.unwrap_or_default())
    } else {
        distribs.ok_or_else(|| Error::UnknownNodejsVersion(version_str.to_string()))
    }
}
