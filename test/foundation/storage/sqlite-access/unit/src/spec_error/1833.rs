#[test]
fn test_spec_error() {
    #[derive(Debug)]
    struct DebugError;

    struct AnyError;

    let _e: Box<dyn Error + Send + Sync + 'static> =
        __spec_error!(std::io::Error::from(std::io::ErrorKind::Unsupported));

    let _e: Box<dyn Error + Send + Sync + 'static> = __spec_error!("displayable error");

    let _e: Box<dyn Error + Send + Sync + 'static> = __spec_error!(DebugError);

    let _e: Box<dyn Error + Send + Sync + 'static> = __spec_error!(AnyError);

    let _e: Box<dyn Error + Send + Sync + 'static> = __spec_error!(&1i32);
}
