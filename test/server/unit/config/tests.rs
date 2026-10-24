use super::*;
#[test]
fn runtime_has_one_root_and_rejects_retired_proxy_configuration() {
    assert_eq!(
        Settings::default().data_dir,
        PathBuf::from("/opt/isarmg/blog/state")
    );
    for field in ["trusted_proxies", "development_http"] {
        let mut document = serde_json::to_value(Settings::default()).unwrap();
        document[field] = serde_json::json!(true);
        assert!(serde_json::from_value::<Settings>(document).is_err());
    }
}
#[test]
fn selected_root_controls_default_media_and_private_namespace_validation() {
    let root = PathBuf::from("/srv/test-blog");
    let settings = Settings {
        root: root.clone(),
        data_dir: root.join("state"),
        ..Default::default()
    };
    assert_eq!(settings.media_path().unwrap(), root.join("media"));
    settings.validate().unwrap();
    for namespace in [
        "bin",
        "libexec",
        "share",
        "config",
        "logs",
        "run",
        "maintenance",
    ] {
        let invalid = Settings {
            media: Some(root.join(namespace).join("public")),
            ..settings.clone()
        };
        assert!(invalid.validate().is_err());
        let invalid = Settings {
            data_dir: root.join(namespace).join("state"),
            ..settings.clone()
        };
        assert!(invalid.validate().is_err());
    }
    let external = Settings {
        media: Some(PathBuf::from("/srv/test-blog-media")),
        data_dir: PathBuf::from("/srv/test-blog-data"),
        ..settings
    };
    external.validate().unwrap();
}

#[test]
fn development_requires_loopback() {
    for address in ["10.20.0.12:8081", "0.0.0.0:8081", "[::]:8081"] {
        let mut value = Settings {
            bind: address.parse().unwrap(),
            ..Default::default()
        };
        value.validate().unwrap();
        value.development = true;
        assert!(value.validate().is_err());
    }
}

#[path = "../../../../src/server/src/installation_definition.rs"]
mod installation_definition;

#[test]
fn current_installation_plan_retains_runtime_defaults_and_database_location() {
    let destination = tempfile::tempdir().unwrap();
    let identity = blog_application::contracts::ReleaseIdentity {
        product: "blog".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        source_revision: "a".repeat(40),
        target: "x86_64-unknown-linux-gnu".into(),
        state_contract_sha256: "0".repeat(64),
    };
    let schema = crate::schema::current_identity().unwrap();
    installation_definition::emit(
        destination.path(),
        &identity,
        &schema,
        include_str!(concat!(env!("OUT_DIR"), "/blog-current-schema.sql")),
    );
    let material: serde_json::Value = serde_json::from_slice(
        &std::fs::read(destination.path().join("contracts/install.json")).unwrap(),
    )
    .unwrap();
    // Pin the public defaults and path descriptor, not how the Rust exporter is written.
    assert_eq!(
        material["config"]["defaults"],
        serde_json::json!({
            "data_dir": "$ROOT/state", "log_level": "info", "media": null,
            "bind": "127.0.0.1:8081", "development": false, "web": null
        })
    );
    assert_eq!(material["databases"][0]["name"], "site");
    assert_eq!(
        material["databases"][0]["path"],
        serde_json::json!({
            "base": "config", "value": "/data_dir", "fallback": null,
            "suffix": "site.sqlite", "encoding": "path"
        })
    );
    assert_eq!(material["secrets"], serde_json::json!([]));

    let root = Path::new("/srv/current-blog");
    let settings = Settings::for_root(root);
    let directory = material["config"]["defaults"]["data_dir"]
        .as_str()
        .unwrap()
        .replace("$ROOT", root.to_str().unwrap());
    let suffix = material["databases"][0]["path"]["suffix"].as_str().unwrap();
    assert_eq!(
        settings.database_path().unwrap(),
        Path::new(&directory).join(suffix)
    );
    assert_eq!(
        settings.database_path().unwrap(),
        root.join("state/site.sqlite")
    );
    assert_eq!(settings.media_path().unwrap(), root.join("media"));
    assert_eq!(
        serde_json::to_value(Settings::default()).unwrap(),
        serde_json::json!({
            "data_dir": "/opt/isarmg/blog/state", "log_level": "info", "media": null,
            "bind": "127.0.0.1:8081", "development": false, "web": null
        })
    );
}
