//! Build-time export of product-owned current installation data.
use blog_application::contracts::{ReleaseIdentity, SchemaIdentity};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn root_path(value: &str) -> Value {
    json!({"base":"root","value":value,"fallback":null,"suffix":"","encoding":"path"})
}
fn asset(directory: &Path, name: &str, bytes: &[u8]) -> Value {
    std::fs::write(directory.join(name), bytes).expect("write generated current material");
    json!({"path":format!("share/contracts/{name}"),"sha256":sha(bytes)})
}
fn relocate_defaults(value: &mut Value) {
    match value {
        Value::String(text) => {
            if let Some(suffix) = text.strip_prefix("/opt/isarmg/blog/") {
                *text = format!("$ROOT/{suffix}");
            }
        }
        Value::Object(values) => values.values_mut().for_each(relocate_defaults),
        Value::Array(values) => values.iter_mut().for_each(relocate_defaults),
        _ => {}
    }
}

pub(crate) fn emit(
    destination: &Path,
    identity: &ReleaseIdentity,
    schema: &SchemaIdentity,
    ddl: &str,
) {
    let directory = destination.join("contracts");
    std::fs::create_dir_all(&directory).expect("create immutable material directory");
    let mut material = blog_application::installation_material::common(identity, schema, None)
        .expect("generate authoritative public installation material");
    let definitions = material
        .as_object_mut()
        .unwrap()
        .remove("definitions")
        .unwrap();
    asset(
        &directory,
        "common-definitions.json",
        &serde_json::to_vec(&definitions).unwrap(),
    );
    let mut database = material
        .as_object_mut()
        .unwrap()
        .remove("database")
        .unwrap();
    database["name"] = json!("site");
    let mut path = crate::config_definition::data_path_spec();
    path["suffix"] = json!("site.sqlite");
    database["path"] = path;
    database["ddl"] = asset(&directory, "current-schema.sql", ddl.as_bytes());
    database["initialize"].as_array_mut().unwrap().insert(0, json!({
        "sql":include_str!("../schema/initialize-site-author.sql"),"parameters":[],"expected_changes":1
    }));
    material["databases"] = json!([database]);
    let mut config =
        blog_application::configuration_contract::generate::<crate::config_definition::Settings>()
            .expect("derive current runtime configuration shape");
    config["schema"].as_object_mut().unwrap().remove("required");
    relocate_defaults(&mut config["defaults"]);
    config["data_dir"] = crate::config_definition::data_path_spec();
    config["bind_pointer"] = json!("/bind");
    config["rules"] = crate::config_definition::current_rules(true);
    let baseline = config["defaults"].clone();
    let mut cases = vec![
        json!({"name":"current-defaults","value":baseline,"valid":true}),
        json!({"name":"omitted-values-use-current-defaults","value":{},"valid":true}),
    ];
    for (name, field, value, valid) in [
        (
            "unknown-property",
            "removed_timezone",
            json!("local"),
            false,
        ),
        ("unknown-log-level", "log_level", json!("invalid"), false),
        ("invalid-bind", "bind", json!("not-a-socket"), false),
        ("private-data-required", "data_dir", Value::Null, false),
        ("relative-media", "media", json!("relative"), false),
        (
            "protected-namespace",
            "data_dir",
            json!("$ROOT/maintenance/data"),
            false,
        ),
        (
            "business-subtree",
            "media",
            json!("$ROOT/state/archive/videos"),
            true,
        ),
        (
            "business-exposes-database",
            "media",
            json!("$ROOT/state/site.sqlite.private"),
            false,
        ),
        (
            "business-exposes-locks",
            "media",
            json!("$ROOT/state/.state-instance.lock"),
            false,
        ),
        (
            "business-contains-private-state",
            "media",
            json!("$ROOT"),
            false,
        ),
        (
            "bound-release-external-web",
            "web",
            json!("$ROOT/external-web"),
            false,
        ),
    ] {
        let mut value_case = config["defaults"].clone();
        value_case[field] = value;
        cases.push(json!({"name":name,"value":value_case,"valid":valid}));
    }
    let mut bad_development = config["defaults"].clone();
    bad_development["development"] = json!(true);
    bad_development["bind"] = json!("0.0.0.0:8081");
    cases
        .push(json!({"name":"development-must-be-loopback","value":bad_development,"valid":false}));
    config["cases"] = json!(cases);
    material["config"] = config;
    material["secrets"] = json!([]);
    let state = crate::config_definition::data_path_spec();
    let media = crate::config_definition::media_path_spec();
    let mut instance_lock = state.clone();
    instance_lock["suffix"] = json!(".state-instance.lock");
    material["resources"] = json!({
        "state_paths":[state,root_path("config"),media],
        "writable_paths":[state,media,root_path("logs"),root_path("run")],
        "readonly_paths":[root_path("config/config.json")],
        "shared_paths":[],"business_paths":[media],"business_directories":[],"config_write_paths":[],
        "private_directories":[state,root_path("logs"),root_path("run")],
        "runtime_locks":[{"path":instance_lock,"kind":"file"}]
    });
    material["writers"] = json!([{"role":"server","service":"blog.service",
        "configuration_paths":[root_path("config/config.json")]}]);
    material["media"] = Value::Null;
    for (path, bytes) in [
        (
            "src/server/src/schema_definition.rs",
            include_bytes!("schema_definition.rs").as_slice(),
        ),
        (
            "src/server/src/config_definition.rs",
            include_bytes!("config_definition.rs").as_slice(),
        ),
        (
            "src/server/src/installation_definition.rs",
            include_bytes!("installation_definition.rs").as_slice(),
        ),
        (
            "src/server/schema/product.sql",
            include_bytes!("../schema/product.sql").as_slice(),
        ),
        (
            "src/server/schema/initialize-site-author.sql",
            include_bytes!("../schema/initialize-site-author.sql").as_slice(),
        ),
    ] {
        material["authority_sources"]
            .as_array_mut()
            .unwrap()
            .push(json!({"path":path,
            "sha256":sha(bytes),
            "generator":format!("blog/src/server/build.rs@{}",identity.source_revision)}));
    }
    asset(
        &directory,
        "install.json",
        &serde_json::to_vec(&material).unwrap(),
    );
}
