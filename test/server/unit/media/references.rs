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
        .filter_map(|event| {
            if let Event::Start(Tag::Image { dest_url, .. }) = event {
                Some(dest_url.into_string())
            } else {
                None
            }
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

#[test]
fn browser_normalized_url_paths_retain_media_across_document_containers() {
    let path = format!("/media/{}.png", "a".repeat(64));
    for alias in [
        path.replace("/media/", "/media/./"),
        path.replace("/media/", "/media/nested/../"),
        path.replace("/media/", "/media/%2e/"),
        path.replace("/media/", "/media/nested/%2E%2e/"),
        format!(
            "https://site.example{}",
            path.replace("/media/", "/media/./")
        ),
    ] {
        for document in [
            alias.clone(),
            format!("![saved]({alias})"),
            format!("![saved][upload]\n\n[upload]: {alias}"),
            serde_json::to_string(&serde_json::json!({"logo":alias})).unwrap(),
        ] {
            assert!(contains_reference(&document, &path), "{document}");
            assert!(
                contains_reference(&format!("unrelated\0{document}\0another field"), &path),
                "SQL field boundaries: {document}"
            );
        }
    }
    let alias = path.replace("/media/", "/media/./");
    assert!(contains_reference(
        &format!("Visit https://site.example{alias} today"),
        &path
    ));
    assert!(contains_reference(
        &format!("Visit site.example{alias}?next=https://other.example today"),
        &path
    ));
    assert!(!contains_reference(
        &format!("/elsewhere/{}.png", "a".repeat(64)),
        &path
    ));
    assert!(!contains_reference(
        &format!("![not resolved][upload]\0[upload]: {alias}"),
        &path
    ));
}

#[test]
fn video_fences_retain_browser_normalized_media_without_activating_other_code() {
    let path = format!("/media/{}.webm", "b".repeat(64));
    for alias in [
        path.clone(),
        path.replace("/media/", "/media/./"),
        path.replace("/media/", "/media/nested/../"),
        path.replace("/media/", "/media/%2e/"),
        path.replace("/media/", "/media/nested/%2E%2e/"),
        format!(
            "https://site.example{}?download=1#t=2",
            path.replace("/media/", "/media/./")
        ),
    ] {
        for source in [
            format!("```video\n{alias}\n```"),
            format!("Before\n\n~~~video \n\n{alias}\n\n~~~\n\nAfter"),
            format!("```rust\nexample\n```\n\n```video\n{alias}\n```"),
        ] {
            assert!(contains_reference(&source, &path), "{source}");
            assert!(
                contains_reference(&format!("\0{source}\0"), &path),
                "SQL field boundaries: {source}"
            );
            assert!(
                contains_reference(&serde_json::to_string(&source).unwrap(), &path),
                "JSON document: {source}"
            );
        }
    }

    // These aliases occur only in inert examples. Treating all code-block text
    // as an active destination would change ordinary Markdown retention rules.
    let alias = path.replace("/media/", "/media/./");
    for source in [
        format!("```rust\n{alias}\n```"),
        format!("```VIDEO\n{alias}\n```"),
        format!("```video example\n{alias}\n```"),
        format!("````markdown\n```video\n{alias}\n```\n````"),
        format!("`{alias}`"),
    ] {
        assert!(!contains_reference(&source, &path), "{source}");
    }
}
