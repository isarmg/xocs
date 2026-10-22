//! Current configuration data definition shared by runtime parsing and build-time contract generation.
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(skip)]
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub log_level: String,
    pub media: Option<PathBuf>,
    pub bind: SocketAddr,
    pub development: bool,
    pub web: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/opt/isarmg/blog"),
            data_dir: PathBuf::from("/opt/isarmg/blog/state"),
            log_level: "info".into(),
            media: None,
            bind: "127.0.0.1:8081".parse().expect("static loopback address"),
            development: false,
            web: None,
        }
    }
}

/// Resolve only declared current resources; these descriptors also drive the
/// independent installer, so runtime and tooling share one path authority.
pub(crate) fn data_path_spec() -> serde_json::Value {
    serde_json::json!({"base":"config","value":"/data_dir","fallback":null,"suffix":"","encoding":"path"})
}
pub(crate) fn media_path_spec() -> serde_json::Value {
    serde_json::json!({"base":"config","value":"/media","fallback":{"base":"root","value":"media","fallback":null,"suffix":"","encoding":"path"},"suffix":"","encoding":"path"})
}
pub(crate) fn current_rules(source_bound: bool) -> serde_json::Value {
    use serde_json::json;
    let protected = blog_application::server_layout::ROOT_RESERVED_DIRECTORIES;
    json!([
        {"kind":"enum","fields":["/log_level"],"arguments":{"values":["off","error","warn","info","debug","trace"]}},
        {"kind":"socket-address","fields":["/bind"],"arguments":{"loopback_when":"/development"}},
        {"kind":"path","fields":["/data_dir"],"arguments":{"absolute":true}},
        {"kind":"path","fields":["/media"],"arguments":{"absolute":true,"optional":true}},
        {"kind":"path","fields":["/web"],"arguments":{"absolute":true,"optional":true}},
        {"kind":"root-state-paths","fields":[],"arguments":{"paths":[data_path_spec()],"protected":protected}},
        {"kind":"root-state-paths","fields":[],"arguments":{"paths":[media_path_spec()],"protected":protected}},
        {"kind":"path-relation","fields":[],"arguments":{"parent":data_path_spec(),"child":media_path_spec(),"allow_child_subtree":true,"forbidden_child_first_component_prefixes":["site.sqlite",".state-"]}},
        {"kind":"external-web-override","fields":["/web"],"arguments":{"development_pointer":"/development","source_bound":source_bound}}
    ])
}
