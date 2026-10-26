use super::*;
use anyhow::Result;

#[test]
fn compiled_builder_binds_every_source_file() -> Result<()> {
    let package = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository = package
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap();
    let compiled = compiled_inputs()
        .into_iter()
        .map(|(name, digest)| (name.to_owned(), digest))
        .collect::<inventory::Files>();
    for path in inventory::walk(&package.join("src"))? {
        let name = inventory::relative(path.strip_prefix(repository)?)?;
        assert_eq!(
            compiled.get(&name),
            Some(&inventory::sha(&inventory::read(&path)?)),
            "Builder implementation is missing from compiled provenance: {name}",
        );
    }
    Ok(())
}

#[test]
fn failed_publish_preserves_previous_pair() -> Result<()> {
    let dir = tempfile::tempdir()?;
    for name in ["pkg", "dist", "stage-pkg", "backups"] {
        fs::create_dir(dir.path().join(name))?;
    }
    fs::write(dir.path().join("pkg/old"), "old package")?;
    fs::write(dir.path().join("dist/old"), "old dist")?;
    fs::write(dir.path().join("pkg/blog-assets.json"), "{}")?;
    fs::write(dir.path().join("dist/blog-assets.json"), "{}")?;
    let receipt = dir.path().join("receipt.json");
    let owner = BuildLock::acquire(dir.path())?;
    let result = replace_pair(
        &owner,
        &dir.path().join("pkg"),
        &dir.path().join("stage-pkg"),
        &dir.path().join("dist"),
        &dir.path().join("missing-dist"),
        &dir.path().join("backups"),
        Some((&receipt, b"new receipt")),
    );
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(dir.path().join("pkg/old"))?,
        "old package"
    );
    assert_eq!(fs::read_to_string(dir.path().join("dist/old"))?, "old dist");
    assert!(!receipt.exists());
    assert_eq!(fs::read_dir(dir.path())?.count(), 5);
    Ok(())
}

fn publication_fixture() -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    for name in ["pkg", "dist", "stage-pkg", "stage-dist", "backups"] {
        let path = dir.path().join(name);
        fs::create_dir(&path)?;
        if name != "backups" {
            fs::write(path.join("contents"), name)?;
            fs::write(path.join("blog-assets.json"), "{}")?;
        }
    }
    Ok(dir)
}

#[test]
fn receipt_collision_rolls_back_both_outputs_and_preserves_existing_receipt() -> Result<()> {
    let dir = publication_fixture()?;
    let root = dir.path();
    let receipt = root.join("receipt.json");
    // This models another producer creating the receipt after the build's
    // early check but before the final create-new publication.
    fs::write(&receipt, "another producer's receipt")?;
    let owner = BuildLock::acquire(root)?;
    let result = replace_pair(
        &owner,
        &root.join("pkg"),
        &root.join("stage-pkg"),
        &root.join("dist"),
        &root.join("stage-dist"),
        &root.join("backups"),
        Some((&receipt, b"new receipt")),
    );
    assert!(result.is_err());
    for name in ["pkg", "dist", "stage-pkg", "stage-dist"] {
        assert_eq!(fs::read_to_string(root.join(name).join("contents"))?, name);
    }
    assert_eq!(fs::read_to_string(&receipt)?, "another producer's receipt");
    drop(result);
    assert_eq!(fs::read_dir(root)?.count(), 7);
    Ok(())
}

#[test]
fn successful_publication_includes_complete_private_receipt() -> Result<()> {
    let dir = publication_fixture()?;
    let root = dir.path();
    let receipt = root.join("receipt.json");
    let owner = BuildLock::acquire(root)?;
    replace_pair(
        &owner,
        &root.join("pkg"),
        &root.join("stage-pkg"),
        &root.join("dist"),
        &root.join("stage-dist"),
        &root.join("backups"),
        Some((&receipt, b"complete receipt\n")),
    )?;
    assert_eq!(fs::read_to_string(root.join("pkg/contents"))?, "stage-pkg");
    assert_eq!(
        fs::read_to_string(root.join("dist/contents"))?,
        "stage-dist"
    );
    assert_eq!(fs::read(&receipt)?, b"complete receipt\n");
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let metadata = fs::metadata(receipt)?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(metadata.nlink(), 1);
    }
    Ok(())
}

#[test]
fn receipt_cannot_overlap_output_directories_through_path_aliases() -> Result<()> {
    let dir = publication_fixture()?;
    let root = dir.path();
    for relative in [
        "pkg/receipt.json",
        "dist/receipt.json",
        "pkg/../dist/receipt.json",
        "dist/../pkg/receipt.json",
        "pkg",
        "dist",
    ] {
        let receipt = root.join(relative);
        assert!(
            receipt_destination(&receipt, &root.join("pkg"), &root.join("dist")).is_err(),
            "{relative}"
        );
        assert_eq!(fs::read_to_string(root.join("pkg/contents"))?, "pkg");
        assert_eq!(fs::read_to_string(root.join("dist/contents"))?, "dist");
    }
    Ok(())
}

#[test]
fn a_competing_producer_cannot_enter_build_or_replace_an_owned_pair() -> Result<()> {
    use std::sync::mpsc;

    let fixture = publication_fixture()?;
    let root = fixture.path();
    for output in ["pkg", "dist"] {
        fs::write(root.join(output).join("contents"), "old generation")?;
    }
    for output in ["stage-pkg", "stage-dist"] {
        fs::write(root.join(output).join("contents"), "new generation")?;
    }
    let (entered, ready) = mpsc::channel();
    let (resume, resumed) = mpsc::channel();
    let product = root.to_owned();
    let first = std::thread::spawn(move || -> Result<()> {
        let owner = BuildLock::acquire(&product)?;
        entered.send(())?;
        resumed.recv()?;
        Ok(replace_pair(
            &owner,
            &product.join("pkg"),
            &product.join("stage-pkg"),
            &product.join("dist"),
            &product.join("stage-dist"),
            &product.join("backups"),
            None,
        )?)
    });
    ready.recv_timeout(std::time::Duration::from_secs(5))?;
    let options = Options {
        root: root.into(),
        offline: true,
        debug: true,
        dist: None,
    };
    let competing = build(&options).unwrap_err();
    assert!(matches!(
        competing,
        BuildError::Lock(LockError::Busy { .. })
    ));
    for output in ["pkg", "dist"] {
        assert_eq!(
            fs::read_to_string(root.join(output).join("contents"))?,
            "old generation"
        );
    }
    resume.send(())?;
    first.join().expect("first producer panicked")?;
    for output in ["pkg", "dist"] {
        assert_eq!(
            fs::read_to_string(root.join(output).join("contents"))?,
            "new generation"
        );
    }
    assert!(root.join("target/.blog-assets/build.lock").is_file());
    let next = BuildLock::acquire(root)?;
    next.validate(root)?;
    Ok(())
}

#[test]
fn build_ownership_cannot_be_used_for_another_product_root() -> Result<()> {
    let first = tempfile::tempdir()?;
    let other = tempfile::tempdir()?;
    let owner = BuildLock::acquire(first.path())?;
    let options = Options {
        root: other.path().into(),
        offline: true,
        debug: true,
        dist: None,
    };
    let error = build_locked(&options, &owner).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("lock belongs to another product root")
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn unsafe_lock_files_are_rejected_without_repair_or_mutation() -> Result<()> {
    use std::os::unix::fs::{MetadataExt, symlink};
    let repository = tempfile::tempdir()?;
    let external = tempfile::NamedTempFile::new()?;
    fs::write(external.path(), "unrelated bytes")?;
    let directory = repository.path().join("target/.blog-assets");
    fs::create_dir_all(&directory)?;
    let lock = directory.join("build.lock");
    symlink(external.path(), &lock)?;
    assert!(BuildLock::acquire(repository.path()).is_err());
    assert!(fs::symlink_metadata(&lock)?.file_type().is_symlink());
    assert_eq!(fs::read(external.path())?, b"unrelated bytes");
    fs::remove_file(&lock)?;
    fs::hard_link(external.path(), &lock)?;
    assert!(BuildLock::acquire(repository.path()).is_err());
    assert_eq!(fs::metadata(&lock)?.nlink(), 2);
    assert_eq!(fs::read(external.path())?, b"unrelated bytes");
    Ok(())
}

#[test]
fn library_failures_keep_the_public_thread_safe_error_contract() {
    fn error_type<T: std::error::Error + Send + Sync + 'static>() {}
    error_type::<BuildError>();
    error_type::<LockError>();
    error_type::<DistributionError>();
    error_type::<PublishError>();
    error_type::<BrowserModuleError>();
    error_type::<ProcessError>();
    error_type::<assets::AssetError>();
    error_type::<javascript::JavaScriptError>();
    error_type::<inventory::InventoryError>();
    error_type::<graph::GraphError>();
    error_type::<dev::DevError>();
    error_type::<dev::ProxyError>();
    error_type::<server_build::NativeBuildError>();
}

#[test]
fn non_unicode_distribution_is_rejected_before_artifact_publication() -> Result<()> {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir()?;
    let output = root.path().join(std::ffi::OsString::from_vec(vec![0xff]));
    let options = Options {
        root: root.path().to_owned(),
        offline: true,
        debug: false,
        dist: Some(output.clone()),
    };
    assert!(
        matches!(distribution(&options), Err(DistributionError::NonUnicodeOutput(path)) if path == output)
    );
    assert!(!output.exists());
    Ok(())
}

#[test]
fn distribution_checks_authored_paths_against_the_physical_product_root() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("product");
    fs::create_dir(&root)?;
    fs::create_dir(fixture.path().join("alias"))?;
    for directory in ["src", "docs", "test", "compliance"] {
        fs::create_dir(root.join(directory))?;
    }
    let aliased_root = fixture.path().join("alias/../product");
    for directory in ["src", "docs", "test", "compliance"] {
        let options = Options {
            root: aliased_root.clone(),
            offline: true,
            debug: false,
            dist: Some(root.join(directory)),
        };
        assert!(
            matches!(
                distribution(&options),
                Err(DistributionError::AuthoredDirectory)
            ),
            "an aliased root must still protect {directory}"
        );
    }
    let options = Options {
        root: aliased_root,
        offline: true,
        debug: false,
        dist: Some(root.join("generated")),
    };
    assert_eq!(distribution(&options)?, root.join("generated"));
    Ok(())
}
