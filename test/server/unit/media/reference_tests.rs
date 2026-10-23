use super::*;

#[test]
fn commonmark_reference_detection_preserves_escape_context() {
    let hash = "a".repeat(64);
    let path = format!("/media/{hash}.png");
    let escaped = path.replace('/', "\\/").replace('.', "\\.");
    let double_escaped = format!("/media/{hash}\\\\.png");
    let illegal_escape = format!("/media/\\a{}.png", "a".repeat(63));
    for (source, expected_destination, referenced) in [
        (format!("![saved]({escaped})"), path.clone(), true),
        (
            format!("![saved][upload]\n\n[upload]: {escaped}"),
            path.clone(),
            true,
        ),
        (
            format!("![other]({double_escaped})"),
            format!("/media/{hash}\\.png"),
            false,
        ),
        (format!("![other]({illegal_escape})"), illegal_escape, false),
    ] {
        let destinations: Vec<_> = Parser::new_ext(
            &source,
            Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
        )
        .filter_map(|event| match event {
            Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.into_string()),
            _ => None,
        })
        .collect();
        assert_eq!(destinations, [expected_destination], "{source}");
        assert_eq!(contains_reference(&source, &path), referenced, "{source}");
        assert_eq!(
            contains_reference(&format!("\0\0{source}\0"), &path),
            referenced,
            "SQL-separated fields: {source}"
        );
        assert_eq!(
            contains_reference(&serde_json::to_string(&vec![&source]).unwrap(), &path),
            referenced,
            "JSON-encoded Markdown: {source}"
        );
    }
    assert!(!contains_reference(
        &format!("`![example]({escaped})`"),
        &path
    ));
}
