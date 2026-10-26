#[path = "src/config_definition.rs"]
#[allow(dead_code)] // Runtime-only fields are intentionally absent from exported configuration.
mod config_definition;
#[path = "src/installation_definition.rs"]
mod installation_definition;
#[path = "../tools/assets/src/platform_contract.rs"]
mod platform_contract;
#[path = "src/schema_definition.rs"]
mod schema_definition;
use std::{env, fs, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    platform_contract::check();
    println!("cargo:rerun-if-env-changed=BLOG_WEB_DIST");
    println!("cargo:rerun-if-env-changed=BLOG_BUILD_METADATA_DIR");
    println!("cargo:rerun-if-env-changed=BLOG_SOURCE_REVISION");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repository root")
        .to_path_buf();
    let source_digest = core_source_sha256(&root.join("src/application"))?;
    println!("cargo:rustc-env=BLOG_CORE_SOURCE_SHA256={source_digest}");
    let web = env::var_os("BLOG_WEB_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("src/web/dist"));
    let target = env::var("TARGET").expect("Cargo build target");
    let revision = match env::var("BLOG_SOURCE_REVISION") {
        Ok(revision) => revision,
        Err(env::VarError::NotPresent) => "unbound".into(),
        Err(error @ env::VarError::NotUnicode(_)) => return Err(error.into()),
    };
    if revision != "unbound"
        && !(revision.len() == 40
            && revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')))
    {
        return Err("BLOG_SOURCE_REVISION must be unbound or a full lowercase Git commit".into());
    }
    println!("cargo:rustc-env=BLOG_BUILD_TARGET={target}");
    println!("cargo:rustc-env=BLOG_SOURCE_REVISION={revision}");
    println!("cargo:rerun-if-changed=schema/product.sql");
    let schema = format!(
        "{};\n{}\n{}",
        blog_application::schema_identity::PRODUCT_METADATA_DDL,
        blog_application::admin_sqlite::ADMIN_PERSISTENT_DDL,
        include_str!("schema/product.sql")
    );
    std::fs::write(
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"))
            .join("blog-current-schema.sql"),
        &schema,
    )?;
    // This is trusted DDL composed from owned source constants, never input.
    let fingerprint = blog_application::sqlite::fingerprint_trusted_ddl(&[&schema])?;
    println!("cargo:rustc-env=BLOG_SCHEMA_SHA256={fingerprint}");
    blog_application::web_assets::build::generate(web, &fingerprint, &source_digest)?;
    emit_offline_metadata(&fingerprint, &schema, revision)
}

fn emit_offline_metadata(schema_hash: &str, ddl: &str, revision: String) -> Result<()> {
    use blog_application::contracts::{
        StateContract, StateResource, StateResourceKind, StateSchemaIdentity,
    };
    use sha2::{Digest, Sha256};
    let Some(destination) = env::var_os("BLOG_BUILD_METADATA_DIR") else {
        return Ok(());
    };
    let destination = PathBuf::from(destination);
    if !destination.is_absolute() || !fs::metadata(&destination)?.is_dir() {
        return Err("explicit metadata directory required".into());
    }
    // An unbound development build has Web identity and inventory, but cannot
    // truthfully export source-bound installation or release contracts.
    if revision == "unbound" {
        return Ok(());
    }
    let schema = schema_definition::identity(schema_hash).expect("current schema");
    let contract = StateContract {
        contract_version: blog_application::contracts::STATE_CONTRACT_VERSION,
        application: "blog".into(),
        application_version: env!("CARGO_PKG_VERSION").into(),
        source_revision: revision.clone(),
        schema: Some(StateSchemaIdentity {
            revision: schema.schema_revision,
            sha256: schema_hash.into(),
        }),
        maintenance_locks: vec![".state-instance.lock".into()],
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
    let bytes = serde_json::to_vec(&contract)?;
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let identity = blog_application::contracts::ReleaseIdentity {
        product: "blog".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        source_revision: revision,
        target: blog_application::server_target::SERVER_TARGET_TRIPLE.into(),
        state_contract_sha256: digest,
    };
    installation_definition::emit(&destination, &identity, &schema, ddl)?;
    fs::write(
        destination.join("blog-identity.json"),
        serde_json::to_vec(&identity)?,
    )?;
    fs::write(
        destination.join("blog-schema.json"),
        serde_json::to_vec(&schema)?,
    )?;
    fs::write(destination.join("blog-state-contract.json"), bytes)?;
    fs::copy(
        PathBuf::from(env::var_os("OUT_DIR").expect("invariant: Cargo supplies OUT_DIR"))
            .join("blog-browser-assets.rs.manifest.json"),
        destination.join("blog-browser-assets.json"),
    )?;
    Ok(())
}

fn core_source_sha256(directory: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    fn collect(directory: &std::path::Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                collect(&path, files)?;
            } else {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(directory, &mut files)?;
    files.sort_unstable();
    let mut hash = Sha256::new();
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        hash.update(
            path.strip_prefix(directory)
                .expect("invariant: collected source paths are below their source root")
                .to_string_lossy()
                .as_bytes(),
        );
        hash.update([0]);
        hash.update(fs::read(&path)?);
        hash.update([0]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
