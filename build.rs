use std::{collections::BTreeSet, env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=XCSS_WEB_DIST");
    println!("cargo:rerun-if-env-changed=XSSI_BUILD_METADATA_DIR");
    println!("cargo:rerun-if-env-changed=XOCS_SOURCE_REVISION");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let lock_path = root.join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());
    let lock = fs::read_to_string(lock_path).expect("read unique workspace Cargo.lock");
    let revisions = lock
        .lines()
        .filter_map(|line| {
            line.strip_prefix("source = \"git+https://github.com/isarmg/xcss.git?rev=")
        })
        .map(|line| {
            let (requested, actual) = line
                .trim_end_matches('"')
                .split_once('#')
                .expect("complete xcss source identity");
            assert_eq!(requested, actual, "xcss source must match its exact pin");
            assert!(
                actual.len() == 40
                    && actual
                        .bytes()
                        .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')),
                "xcss source must be a full lowercase commit"
            );
            actual.to_owned()
        })
        .collect::<BTreeSet<_>>();
    let development = env::var("PROFILE").is_ok_and(|profile| profile == "debug")
        && env::var("XOCS_SOURCE_REVISION").unwrap_or_else(|_| "unbound".into()) == "unbound"
        && env::var_os("XSSI_BUILD_METADATA_DIR").is_none();
    let common_revision = if revisions.is_empty() && development {
        "unbound".into()
    } else {
        assert_eq!(revisions.len(), 1, "one coherent xcss input is required");
        revisions.into_iter().next().expect("one xcss source")
    };
    println!("cargo:rustc-env=XCSS_REVISION={common_revision}");
    let web = env::var_os("XCSS_WEB_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("web/dist"));
    xcss::web_assets::build::generate(web).expect("build embedded Xocs Web assets");
    let target = env::var("TARGET").expect("Cargo build target");
    let revision = env::var("XOCS_SOURCE_REVISION").unwrap_or_else(|_| "unbound".into());
    assert!(
        revision == "unbound"
            || (revision.len() == 40
                && revision
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))),
        "XOCS_SOURCE_REVISION must be unbound or a full lowercase Git commit"
    );
    println!("cargo:rustc-env=XOCS_BUILD_TARGET={target}");
    println!("cargo:rustc-env=XOCS_SOURCE_REVISION={revision}");
    println!("cargo:rerun-if-changed=schema/product.sql");
    let schema = format!(
        "{};\n{}\n{}",
        xcss::schema_identity::PRODUCT_METADATA_DDL,
        xcss::admin_sqlite::ADMIN_PERSISTENT_DDL,
        include_str!("schema/product.sql")
    );
    std::fs::write(
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"))
            .join("xocs-current-schema.sql"),
        &schema,
    )
    .expect("write current composed schema");
    // This is trusted DDL composed from owned source constants, never input.
    let fingerprint =
        xcss::sqlite::fingerprint_trusted_ddl(&[&schema]).expect("fingerprint current Xocs DDL");
    println!("cargo:rustc-env=XOCS_SCHEMA_SHA256={fingerprint}");
    emit_offline_metadata(&fingerprint);
}

fn emit_offline_metadata(schema_hash: &str) {
    use sha2::{Digest, Sha256};
    use xcss::contracts::{StateContract, StateResource, StateResourceKind, StateSchemaIdentity};
    let Some(destination) = env::var_os("XSSI_BUILD_METADATA_DIR") else {
        return;
    };
    let destination = PathBuf::from(destination);
    assert!(
        destination.is_absolute() && destination.is_dir(),
        "explicit metadata directory required"
    );
    let revision =
        env::var("XOCS_SOURCE_REVISION").expect("source-bound metadata requires source revision");
    let schema = xcss::contracts::SchemaIdentity::new("xocs", "xocs-db-v5", 5, schema_hash)
        .expect("current schema");
    let contract = StateContract {
        contract_version: xcss::contracts::STATE_CONTRACT_VERSION,
        application: "xocs".into(),
        application_version: env::var("CARGO_PKG_VERSION").unwrap(),
        source_revision: revision.clone(),
        schema: Some(StateSchemaIdentity {
            revision: 5,
            sha256: schema_hash.into(),
        }),
        maintenance_locks: vec![
            ".state-instance.lock".into(),
            ".state-maintenance.lock".into(),
        ],
        resources: vec![
            StateResource {
                name: "state".into(),
                kind: StateResourceKind::DataTree,
                required: true,
            },
            StateResource {
                name: "media".into(),
                kind: StateResourceKind::DataTree,
                required: true,
            },
            StateResource {
                name: "config".into(),
                kind: StateResourceKind::Configuration,
                required: true,
            },
            StateResource {
                name: "logs".into(),
                kind: StateResourceKind::DataTree,
                required: true,
            },
        ],
        external_requirements: vec![],
        companion_contracts: vec![],
    };
    contract.validate().expect("state contract");
    let bytes = serde_json::to_vec(&contract).unwrap();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let identity = xcss::contracts::ReleaseIdentity {
        product: "xocs".into(),
        version: env::var("CARGO_PKG_VERSION").unwrap(),
        source_revision: revision,
        target: xcss::server_target::SERVER_TARGET_TRIPLE.into(),
        state_contract_sha256: digest,
    };
    identity.validate().expect("bound release identity");
    fs::write(
        destination.join("xocs-identity.json"),
        serde_json::to_vec(&identity).unwrap(),
    )
    .unwrap();
    fs::write(
        destination.join("xocs-schema.json"),
        serde_json::to_vec(&schema).unwrap(),
    )
    .unwrap();
    fs::write(destination.join("xocs-state-contract.json"), bytes).unwrap();
    fs::copy(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("xcss-web-assets.rs.manifest.json"),
        destination.join("xocs-web-assets.json"),
    )
    .unwrap();
}
