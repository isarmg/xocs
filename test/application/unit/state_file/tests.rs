use super::*;
use std::os::unix::fs::{PermissionsExt, symlink};

fn private_tempdir() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))?;
    Ok(directory)
}

#[test]
fn administration_preserves_service_ownership_without_relaxing_runtime()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::{fs::chown, process::CommandExt};
    use std::process::Command;
    if rustix::process::geteuid().as_raw() != 0 {
        let parent = private_tempdir()?;
        PrivateStateDirectory::open_for_administration(parent.path())?
            .try_instance_lock()?
            .release()?;
        return Ok(());
    }
    let parent = private_tempdir()?;
    fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o755))?;
    let path = parent.path().join("service-state");
    fs::create_dir(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    chown(&path, Some(65534), Some(65534))?;
    assert!(matches!(
        PrivateStateDirectory::open(&path),
        Err(Error::UnexpectedOwner { .. })
    ));
    let directory = PrivateStateDirectory::open_for_administration(&path)?;
    assert_eq!(
        (directory.owner_uid(), directory.owner_gid()),
        (65534, 65534)
    );
    let lock = directory.try_instance_lock()?;
    let metadata = fs::metadata(path.join(INSTANCE_LOCK_FILE))?;
    assert_eq!(
        (
            metadata.uid(),
            metadata.gid(),
            metadata.mode() & 0o7777,
            metadata.nlink()
        ),
        (65534, 65534, 0o600, 1)
    );
    let helper = |uid, denied| -> Result<(), Box<dyn std::error::Error>> {
        let result = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "state_file::tests::administration_uid_helper",
                "--ignored",
            ])
            .env("BLOG_ADMIN_TEST_DATA", &path)
            .env("BLOG_ADMIN_TEST_DENIED", if denied { "yes" } else { "no" })
            .uid(uid)
            .gid(uid)
            .output()?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let output = String::from_utf8_lossy(&result.stdout);
        assert!(
            output.contains("running 1 test") && output.contains("1 passed"),
            "UID helper must execute exactly once: {output}"
        );
        Ok(())
    };
    helper(65534, false)?;
    helper(65533, true)?;
    lock.release()?;
    directory
        .create_file("fixture-private.json")?
        .file()
        .sync_all()?;
    assert_eq!(
        fs::metadata(path.join("fixture-private.json"))?.uid(),
        65534
    );
    let alias = parent.path().join("alias");
    symlink(&path, &alias)?;
    assert!(PrivateStateDirectory::open_for_administration(&alias).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o750))?;
    assert!(PrivateStateDirectory::open_for_administration(&path).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    let existing = path.join("wrong-owner");
    fs::write(&existing, b"retain")?;
    fs::set_permissions(&existing, fs::Permissions::from_mode(0o600))?;
    assert!(directory.create_file("wrong-owner").is_err());
    assert_eq!(fs::metadata(&existing)?.uid(), 0);
    assert_eq!(fs::read(existing)?, b"retain");
    Ok(())
}

#[test]
#[ignore = "real uid helper invoked by administration_preserves_service_ownership_without_relaxing_runtime"]
fn administration_uid_helper() {
    let path = PathBuf::from(std::env::var_os("BLOG_ADMIN_TEST_DATA").unwrap());
    if std::env::var("BLOG_ADMIN_TEST_DENIED").unwrap() == "yes" {
        assert!(PrivateStateDirectory::open(&path).is_err());
        assert!(PrivateStateDirectory::open_for_administration(&path).is_err());
    } else {
        let ordinary = PrivateStateDirectory::open(&path).unwrap();
        assert!(ordinary.try_instance_lock().is_err());
        let administrative = PrivateStateDirectory::open_for_administration(&path).unwrap();
        assert!(administrative.try_instance_lock().is_err());
    }
}

#[test]
fn creates_and_reopens_secure_files() -> Result<(), Box<dyn std::error::Error>> {
    let parent = private_tempdir()?;
    let path = parent.path().join("state");
    let state = PrivateStateDirectory::create(&path)?;
    let file = state.create_file("state.sqlite3")?;
    assert_eq!(file.file().metadata()?.mode() & 0o777, 0o600);
    assert_eq!(
        file.identity(),
        state.open_existing("state.sqlite3")?.identity()
    );
    file.verify_identity()?;
    Ok(())
}

#[test]
fn rejects_relative_public_symlink_and_hardlinked_state() -> Result<(), Box<dyn std::error::Error>>
{
    assert!(matches!(
        PrivateStateDirectory::open("relative"),
        Err(Error::StateDirectoryNotAbsolute { .. })
    ));
    let parent = private_tempdir()?;
    let public = parent.path().join("public");
    fs::create_dir(&public)?;
    fs::set_permissions(&public, fs::Permissions::from_mode(0o755))?;
    assert!(matches!(
        PrivateStateDirectory::open(&public),
        Err(Error::UnexpectedMode { .. })
    ));
    let private = parent.path().join("private");
    fs::create_dir(&private)?;
    fs::set_permissions(&private, fs::Permissions::from_mode(0o700))?;
    let link = parent.path().join("link");
    symlink(&private, &link)?;
    assert!(matches!(
        PrivateStateDirectory::open(&link),
        Err(Error::NotDirectory { .. })
    ));
    let state = PrivateStateDirectory::open(&private)?;
    state.create_file("database")?;
    fs::hard_link(private.join("database"), private.join("copy"))?;
    assert!(matches!(
        state.open_existing("database"),
        Err(Error::UnexpectedHardLinkCount { .. })
    ));
    Ok(())
}

#[test]
fn detects_replacement_and_coordinates_locks() -> Result<(), Box<dyn std::error::Error>> {
    let parent = private_tempdir()?;
    let state = PrivateStateDirectory::create(parent.path().join("state"))?;
    let file = state.create_file("database")?;
    fs::rename(file.path(), state.path().join("old"))?;
    state.create_file("database")?;
    assert!(matches!(
        file.verify_identity(),
        Err(Error::FileIdentityChanged { .. })
    ));
    drop(file);

    let instance = state.try_instance_lock()?;
    assert!(matches!(
        state.try_instance_lock(),
        Err(Error::LockUnavailable {
            kind: LockKind::Instance,
            ..
        })
    ));
    assert!(matches!(
        state.try_instance_lock(),
        Err(Error::LockUnavailable {
            kind: LockKind::Instance,
            ..
        })
    ));
    drop(instance);
    let _instance = state.try_instance_lock()?;
    assert!(matches!(
        state.try_instance_lock(),
        Err(Error::LockUnavailable {
            kind: LockKind::Instance,
            ..
        })
    ));
    Ok(())
}

#[test]
fn held_instance_lock_detects_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let parent = private_tempdir()?;
    let state = PrivateStateDirectory::open(parent.path())?;
    let instance = state.try_instance_lock()?;
    fs::rename(
        state.path().join(INSTANCE_LOCK_FILE),
        state.path().join("old-lock"),
    )?;
    state.create_file(INSTANCE_LOCK_FILE)?;
    assert!(matches!(
        instance.verify_identity(),
        Err(Error::FileIdentityChanged { .. })
    ));
    drop(instance);
    Ok(())
}
#[test]
fn implicit_drop_unlocks_instance_even_when_a_description_alias_survives()
-> Result<(), Box<dyn std::error::Error>> {
    let parent = private_tempdir()?;
    let state = PrivateStateDirectory::open(parent.path())?;
    let instance = state.try_instance_lock()?;
    let alias = instance._instance.file().try_clone()?;
    let before = alias.metadata()?;
    assert!(state.try_instance_lock().is_err());
    drop(instance);
    let replacement = state.try_instance_lock()?;
    drop(alias);
    assert!(state.try_instance_lock().is_err());
    replacement.verify_identity()?;
    let after = fs::metadata(state.path().join(INSTANCE_LOCK_FILE))?;
    assert_eq!(
        (
            before.dev(),
            before.ino(),
            before.uid(),
            before.gid(),
            before.mode()
        ),
        (
            after.dev(),
            after.ino(),
            after.uid(),
            after.gid(),
            after.mode()
        )
    );
    replacement.release()?;
    Ok(())
}

#[test]
fn implicit_instance_drop_unlocks_without_unlinking_state() -> Result<(), Box<dyn std::error::Error>>
{
    let parent = private_tempdir()?;
    let state = PrivateStateDirectory::open(parent.path())?;
    let instance = state.try_instance_lock()?;
    let instance_alias = instance._instance.file().try_clone()?;
    let identity = instance._instance.identity();
    drop(instance);
    let replacement = state.try_instance_lock()?;
    drop(instance_alias);
    assert!(state.try_instance_lock().is_err());
    assert_eq!(replacement._instance.identity(), identity);
    replacement.release()?;
    Ok(())
}

#[test]
fn explicit_unlock_prevents_old_guard_drop_from_releasing_a_reacquired_alias()
-> Result<(), Box<dyn std::error::Error>> {
    let parent = private_tempdir()?;
    let state = PrivateStateDirectory::open(parent.path())?;
    let mut instance = state.try_instance_lock()?;
    let alias = instance._instance.file().try_clone()?;
    instance._instance.unlock()?;
    try_flock(
        &alias,
        FlockOperation::NonBlockingLockExclusive,
        LockKind::Instance,
    )?;
    drop(instance);
    assert!(state.try_instance_lock().is_err());
    alias.unlock()?;
    state.try_instance_lock()?.release()?;
    Ok(())
}

#[test]
fn explicit_release_unlocks_the_original_description_after_its_name_is_replaced()
-> Result<(), Box<dyn std::error::Error>> {
    let parent = private_tempdir()?;
    let state = PrivateStateDirectory::open(parent.path())?;
    let instance = state.try_instance_lock()?;
    let alias = instance._instance.file().try_clone()?;
    let original_identity = instance._instance.identity();
    fs::rename(
        state.path().join(INSTANCE_LOCK_FILE),
        state.path().join("original-lock"),
    )?;
    let replacement = state.try_instance_lock()?;
    instance.release()?;
    // Release uses its owned descriptor. The separately opened replacement
    // retains its lease while the original inode can be locked again.
    try_flock(
        &alias,
        FlockOperation::NonBlockingLockExclusive,
        LockKind::Instance,
    )?;
    assert!(state.try_instance_lock().is_err());
    let old_metadata = fs::metadata(state.path().join("original-lock"))?;
    assert_eq!(
        FileIdentity::from_metadata(&old_metadata),
        original_identity
    );
    replacement.verify_identity()?;
    alias.unlock()?;
    replacement.release()?;
    Ok(())
}
