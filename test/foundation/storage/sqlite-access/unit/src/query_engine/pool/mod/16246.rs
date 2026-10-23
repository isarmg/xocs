#[test]
fn assert_pool_traits() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_clone<T: Clone>() {}

    assert_send_sync::<Pool<crate::sqlite::Sqlite>>();
    assert_clone::<Pool<crate::sqlite::Sqlite>>();
}
