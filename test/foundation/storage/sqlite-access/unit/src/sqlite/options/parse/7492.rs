#[test]
fn it_returns_the_parsed_url() -> Result<(), Error> {
    let url = "sqlite://test.db?mode=rw&cache=shared";
    let options: SqliteConnectOptions = url.parse()?;

    let expected_url = Url::parse(url).unwrap();
    assert_eq!(options.build_url(), expected_url);

    Ok(())
}

#[test]
fn filename_url_roundtrip_preserves_literal_percent_escapes() -> Result<(), Error> {
    for filename in [
        "data%20store.db",
        "data%FF.db",
        "data%25.db",
        "data%.db",
        "/tmp/data%2Fstore.db",
        "/tmp/data%3Fstore?part#fragment.db",
    ] {
        let options = SqliteConnectOptions::new().filename(filename);
        let parsed: SqliteConnectOptions = options.build_url().as_str().parse()?;
        assert_eq!(parsed.filename, options.filename, "{filename}");
    }

    Ok(())
}

#[test]
fn filename_url_roundtrip_preserves_path_and_authority_delimiters() -> Result<(), Error> {
    for filename in [
        "data:store.db",
        "[data].db",
        "data\\store.db",
        "data|store.db",
        "data^store.db",
        "data@store.db",
        "data/../store.db",
        "./data.db",
        "//data/store.db",
        "数据/内容.db",
    ] {
        let options = SqliteConnectOptions::new().filename(filename);
        let parsed: SqliteConnectOptions = options.build_url().as_str().parse()?;
        assert_eq!(parsed.filename, options.filename, "{filename}");
    }

    Ok(())
}

#[test]
fn url_roundtrip_does_not_turn_read_only_access_into_read_write() -> Result<(), Error> {
    let options = SqliteConnectOptions::new()
        .filename("test.db")
        .create_if_missing(true)
        .read_only(true);
    let parsed: SqliteConnectOptions = options.build_url().as_str().parse()?;
    assert!(parsed.read_only);
    assert!(!parsed.create_if_missing);

    Ok(())
}
