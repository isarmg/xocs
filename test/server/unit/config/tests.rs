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
