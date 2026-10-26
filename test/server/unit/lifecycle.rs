use super::*;

#[tokio::test]
async fn closing_state_closes_database_and_releases_the_instance_lock() {
    let root = tempfile::tempdir().unwrap();
    let directory = blog_application::state_file::PrivateStateDirectory::open(root.path()).unwrap();
    let instance = directory.try_instance_lock().unwrap();
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let lifecycle = Lifecycle {
        scope: WorkScope::new(),
        pool: pool.clone(),
        lock: Mutex::new(Some(instance)),
    };
    assert!(directory.try_instance_lock().is_err());

    lifecycle.close_state().await.unwrap();
    assert!(pool.is_closed());
    let reacquired = directory.try_instance_lock().unwrap();
    lifecycle.release_lock().unwrap();
    reacquired.release().unwrap();
}
