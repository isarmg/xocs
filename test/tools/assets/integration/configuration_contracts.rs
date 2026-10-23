//! Public TOML contracts exercised by the asset builder that consumes them.
use toml::{datetime::Datetime, editor::DocumentMut, span::Spanned};

#[test]
fn byte_spans_and_unicode_are_preserved() {
    let input = "name = '中文🦀'\ncount = 7\n";
    let values: std::collections::BTreeMap<String, Spanned<toml::Value>> =
        toml::from_str(input).unwrap();
    let value = &values["name"];
    assert_eq!(&input[value.span()], "'中文🦀'");
    assert_eq!(value.get_ref().as_str(), Some("中文🦀"));
}

#[test]
fn formatting_and_error_offsets_remain_byte_based() {
    let text = "# heading\r\n['中文'] # table\r\na = '🦀' # comment\r\n";
    let document = text.parse::<DocumentMut>().unwrap();
    assert_eq!(document.to_string(), text.replace("\r\n", "\n"));
    let invalid = "name = '🦀'\nvalue = [1,,2]";
    let error = invalid.parse::<DocumentMut>().unwrap_err();
    let span = error.span().unwrap();
    assert!(span.start >= invalid.find("[1,").unwrap());
    assert!(span.end <= invalid.len());
    assert!(invalid.is_char_boundary(span.start));
    assert!(error.to_string().contains("line 2"));
}

#[test]
fn utf8_fast_path_boundaries_reject_malformed_syntax() {
    for input in [
        "x = '\u{0}'",
        "x = \"unterminated🦀",
        "x = 2024-13-01",
        "x = [🦀]",
        "🦀 = 1",
        "x = \"\\uD800\"",
        "x = \"\\U00110000\"",
    ] {
        assert!(input.parse::<DocumentMut>().is_err(), "{input:?}");
    }
    for input in [
        "x = '🦀'",
        "x = '\u{80}'",
        "x = '\u{7ff}'",
        "x = '\u{800}'",
        "x = '\u{10000}'",
        "x = '\u{10ffff}'",
    ] {
        assert_eq!(
            input.parse::<DocumentMut>().unwrap().to_string(),
            format!("{input}\n")
        );
    }
    let malformed = std::hint::black_box([0xf0, 0x28, 0x8c, 0x28]);
    assert!(std::str::from_utf8(&malformed).is_err());
    // The public parser accepts &str, not arbitrary bytes; invalid UTF-8 cannot enter it safely.
}

#[test]
fn datetime_values_roundtrip_through_typed_configuration() {
    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct Release {
        published: Datetime,
    }

    let timestamp = "1979-05-27T07:32:00.123-08:00";
    let value: Datetime = timestamp.parse().unwrap();
    assert_eq!(value.to_string(), timestamp);
    let release: Release = toml::from_str(&format!("published = {timestamp}\n")).unwrap();
    assert_eq!(release.published, value);
    let rendered = toml::to_string(&release).unwrap();
    assert_eq!(toml::from_str::<Release>(&rendered).unwrap(), release);
}
