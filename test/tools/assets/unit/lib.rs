use super::*;

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
    let result = replace_pair(
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
    assert_eq!(fs::read_dir(dir.path())?.count(), 4);
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
    let result = replace_pair(
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
    assert_eq!(fs::read_dir(root)?.count(), 6);
    Ok(())
}

#[test]
fn successful_publication_includes_complete_private_receipt() -> Result<()> {
    let dir = publication_fixture()?;
    let root = dir.path();
    let receipt = root.join("receipt.json");
    replace_pair(
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
