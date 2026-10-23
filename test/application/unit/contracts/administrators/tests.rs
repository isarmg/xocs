use super::*;

#[test]
fn password_retention_requires_omission_not_empty_or_null() {
    let request = |field: &str| {
        serde_json::from_str::<AdministratorAccountRequest>(&format!(
            r#"{{"username":"admin","current_password":"correct horse battery"{field}}}"#
        ))
    };
    assert!(request("").unwrap().new_password.is_none());
    assert!(request(r#", "new_password":"""#).is_err());
    assert!(request(r#", "new_password":null"#).is_err());
    assert!(request(r#", "new_password":"new correct password""#).is_ok());
}
