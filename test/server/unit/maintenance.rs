use super::*;
use std::error::Error as _;

fn release_failure() -> blog_application::state_file::Error {
    blog_application::state_file::Error::LockUnavailable {
        kind: blog_application::state_file::LockKind::Instance,
        source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
    }
}

#[test]
fn maintenance_retains_operation_and_release_failures() {
    let failure =
        finish_operation(Err(Error::AdministratorNotFound), Err(release_failure())).unwrap_err();
    assert_eq!(
        failure.source().unwrap().to_string(),
        Error::AdministratorNotFound.to_string()
    );
    let Error::StateRelease { operation, release } = failure else {
        panic!("both operation and release failures must remain visible");
    };
    assert!(matches!(*operation, Error::AdministratorNotFound));
    let source = release
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);

    let failure = finish_operation(Ok(()), Err(release_failure())).unwrap_err();
    assert!(matches!(failure, Error::State(_)));
    assert!(matches!(
        finish_operation(Err(Error::AdministratorNotFound), Ok(())),
        Err(Error::AdministratorNotFound)
    ));
    finish_operation(Ok(()), Ok(())).unwrap();
}
