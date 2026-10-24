use super::*;

#[test]
fn raw_html_is_text_including_svg_math_and_attribute_payloads() {
    for attack in [
        "<script>alert(1)</script>",
        "<img src=x onerror=alert(1)>",
        "<svg><a xlink:href='javascript:alert(1)'>x</a></svg>",
        "<math><mtext><table><mglyph><style><!--</style><img title='--><img src=1 onerror=alert(1)>'>",
        "<!--><img src=x onerror=alert(1)>",
    ] {
        let rendered = html(attack);
        assert!(!rendered.contains("<script"), "{rendered}");
        assert!(!rendered.contains("<img"), "{rendered}");
        assert!(!rendered.contains("<svg"), "{rendered}");
        assert!(!rendered.contains("<math"), "{rendered}");
        assert!(rendered.contains("&lt;"), "{rendered}");
    }
}

#[test]
fn dangerous_and_obfuscated_destinations_are_not_executable() {
    for attack in [
        "[x](javascript:alert%281%29)",
        "[x](JaVaScRiPt:alert%281%29)",
        "[x](jav&#x61;script:alert%281%29)",
        "[x](java&#x09;script:alert%281%29)",
        "[x](vbscript:msgbox%281%29)",
        "[x](data:text/html;base64,PHNjcmlwdD4=)",
        "![x](data:image/svg+xml;base64,PHN2Zy8+)",
        "![x](javascript:alert%281%29)",
        "[x](file:///etc/passwd)",
    ] {
        let rendered = html(attack);
        assert!(!rendered.contains("href="), "{rendered}");
        assert!(!rendered.contains("src="), "{rendered}");
    }
    for value in [
        "javascript:alert(1)",
        "java\nscript:alert(1)",
        "\tjavascript:alert(1)",
        "data:text/html,x",
        "file:///x",
    ] {
        assert!(!allowed_destination(value, false), "{value}");
    }
}

#[test]
fn links_attributes_and_code_are_escaped_and_never_nested() {
    let rendered = html(
        "[safe](https://example.org/?q=1&x=2 \"title\\\" onmouseover=\\\"bad\")\n\n`<img onerror=x>`\n\n```rust\" onmouseover=\"x\n<script>literal</script>\n```\n\n[https://example.org](https://target.org)\n\n![a \" quote](/media/photo.png)",
    );
    assert!(rendered.contains("href=\"https://example.org/?q=1&amp;x=2\""));
    assert!(!rendered.contains(" onmouseover=\""));
    assert!(!rendered.contains("<script>"));
    assert_eq!(rendered.matches("<a ").count(), 2);
    assert!(rendered.contains("alt=\"a &quot; quote\""));
}

#[test]
fn common_article_features_and_plain_links_remain() {
    let rendered = html(
        "## Chapter **One**\nfirst\nsecond\n\n~~old~~\n\n|a|b|\n|-|-|\n|1|2|\n\nVisit https://example.org/path, www.example.com or test@example.org.\n\n`https://code.example.org`\n\n[local](/sort?q=rust#top)",
    );
    assert!(rendered.contains("<h2>Chapter <strong>One</strong></h2>"));
    assert!(rendered.contains("first<br />\nsecond"));
    assert!(rendered.contains("<del>old</del>"));
    assert!(rendered.contains("<table>"));
    for destination in [
        "https://example.org/path",
        "http://www.example.com",
        "mailto:test@example.org",
        "/sort?q=rust#top",
    ] {
        assert!(
            rendered.contains(&format!("href=\"{destination}\"")),
            "{rendered}"
        );
    }
    assert_eq!(rendered.matches("<a ").count(), 4);
}

#[test]
fn bare_links_with_ports_and_path_colons_receive_an_http_scheme() {
    let rendered = html(
        "Visit example.org:8080/docs, www.example.com/docs/part:one, example.org/search?url=https://example.net or https://example.net:8443/a:b.",
    );
    for destination in [
        "http://example.org:8080/docs",
        "http://www.example.com/docs/part:one",
        "http://example.org/search?url=https://example.net",
        "https://example.net:8443/a:b",
    ] {
        assert!(
            rendered.contains(&format!("href=\"{destination}\"")),
            "{rendered}"
        );
    }
}

#[test]
fn video_fences_render_multiple_native_and_canonical_provider_players() {
    let rendered = html(
        "Before\n\n```video\n/media/clip.webm\n```\n\nMiddle\n\n```video\nhttps://cdn.example.org/video.MP4?token=one&part=two#t=3\n```\n\n```video\nhttps://www.youtube.com/watch?v=dQw4w9WgXcQ&autoplay=1&origin=https://evil.example\n```\n\n```video\nhttps://www.bilibili.com/video/BV1xx411c7mD/?p=2&autoplay=1\n```\n\nAfter",
    );
    assert_eq!(rendered.matches("<video ").count(), 2, "{rendered}");
    assert_eq!(rendered.matches("<iframe ").count(), 2, "{rendered}");
    assert!(rendered.contains("src=\"/media/clip.webm\""));
    assert!(
        rendered.contains("src=\"https://cdn.example.org/video.MP4?token=one&amp;part=two#t=3\"")
    );
    assert!(rendered.contains("src=\"https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ\""));
    assert!(rendered.contains("src=\"https://player.bilibili.com/player.html?bvid=BV1xx411c7mD&amp;page=2&amp;autoplay=0\""));
    assert!(!rendered.contains("evil.example"));
    assert!(!rendered.contains("autoplay=1"));
    assert!(rendered.contains("<p>Before</p>"));
    assert!(rendered.contains("<p>Middle</p>"));
    assert!(rendered.contains("<p>After</p>"));
    assert!(rendered.contains("controls playsinline preload=\"metadata\""));
    assert!(rendered.contains("sandbox=\"allow-scripts allow-same-origin allow-presentation\""));
    assert!(
        rendered.contains("loading=\"lazy\" referrerpolicy=\"strict-origin-when-cross-origin\"")
    );
}

#[test]
fn equivalent_platform_watch_short_and_embed_addresses_are_normalized() {
    let youtube = video_html("https://www.youtube.com/watch?v=dQw4w9WgXcQ", None).unwrap();
    for source in [
        "https://youtu.be/dQw4w9WgXcQ?si=tracking",
        "https://www.youtube.com/embed/dQw4w9WgXcQ?autoplay=1",
        "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ",
        "https://m.youtube.com/shorts/dQw4w9WgXcQ",
        "https://www.youtube.com/live/dQw4w9WgXcQ",
    ] {
        assert_eq!(
            video_html(source, None).as_deref(),
            Some(youtube.as_str()),
            "{source}"
        );
    }
    let bilibili = video_html("https://www.bilibili.com/video/BV1xx411c7mD?p=2", None).unwrap();
    assert_eq!(
        video_html(
            "https://player.bilibili.com/player.html?bvid=BV1xx411c7mD&page=2&autoplay=1",
            None
        )
        .as_deref(),
        Some(bilibili.as_str())
    );
    assert!(
        video_html("https://m.bilibili.com/video/av170001", None)
            .unwrap()
            .contains("?aid=170001&amp;autoplay=0")
    );
    assert!(
        video_html("https://player.bilibili.com/player.html?aid=170001", None)
            .unwrap()
            .contains("?aid=170001&amp;autoplay=0")
    );
}

#[test]
fn invalid_video_fences_remain_literal_and_never_create_a_player() {
    for source in [
        "javascript:alert(1)",
        "data:video/mp4;base64,AAAA",
        "//cdn.example.org/clip.mp4",
        "/\\evil.example/clip.mp4",
        "https://user:secret@cdn.example.org/clip.mp4",
        "https://cdn.example.org/audio.mp3",
        "https://cdn.example.org/video",
        "https://www.youtube.com.evil.example/watch?v=dQw4w9WgXcQ",
        "https://www.youtube.com:8443/watch?v=dQw4w9WgXcQ",
        "https://youtu.be/invalid-id",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&v=abcdefghijk",
        "https://player.bilibili.com/player.html?bvid=bad",
        "https://www.bilibili.com/video/BV1xx411c7mD?p=0",
        "https://www.bilibili.com/video/BV1xx411c7mD?p=2&p=3",
        "/media/clip.mp4\nhttps://evil.example",
        "<iframe src=\"https://evil.example\"></iframe>",
    ] {
        let rendered = html(&format!("```video\n{source}\n```"));
        assert!(!rendered.contains("<video "), "{source}: {rendered}");
        assert!(!rendered.contains("<iframe "), "{source}: {rendered}");
        assert!(
            rendered.contains("<pre><code class=\"language-video\">"),
            "{source}: {rendered}"
        );
    }
}

#[test]
fn video_syntax_is_explicit_and_raw_html_still_cannot_execute() {
    let rendered = html(
        "[video](/media/clip.mp4)\n\n![image](/media/clip.mp4)\n\n```rust\n```video\n/media/clip.mp4\n```\n\n~~~text\n```video\n/media/clip.webm\n```\n~~~\n\n```video unsupported-options\n/media/clip.mp4\n```\n\n<video controls src=\"/media/clip.mp4\" onerror=\"alert(1)\"></video>\n\n<iframe src=\"https://www.youtube.com/embed/dQw4w9WgXcQ\"></iframe>",
    );
    assert!(!rendered.contains("<video "), "{rendered}");
    assert!(!rendered.contains("<iframe "), "{rendered}");
    assert!(rendered.contains("href=\"/media/clip.mp4\""));
    assert!(rendered.contains("<img src=\"/media/clip.mp4\""));
    assert!(rendered.contains("&lt;video"));
    assert!(rendered.contains("&lt;iframe"));
}

#[test]
fn video_attachments_escape_urls_and_admit_only_safe_posters() {
    let rendered = video_html(
        "https://cdn.example.org/a.mp4?q=%22&x=%3Cscript%3E",
        Some("/media/poster.png?a=1&b=2"),
    )
    .unwrap();
    assert!(rendered.contains("poster=\"/media/poster.png?a=1&amp;b=2\""));
    assert!(rendered.contains("src=\"https://cdn.example.org/a.mp4?q=%22&amp;x=%3Cscript%3E\""));
    assert!(
        !video_html("/media/a.webm", Some("javascript:alert(1)"))
            .unwrap()
            .contains("poster=")
    );
    assert!(
        video_html("/media/a.webm", None)
            .unwrap()
            .contains("src=\"/media/a.webm\"")
    );
}

#[test]
fn editor_video_blocks_cannot_inject_another_fence_or_html() {
    assert_eq!(
        video_block(" /media/a.webm ").as_deref(),
        Some("```video\n/media/a.webm\n```")
    );
    for source in [
        "/media/a.webm\n```\n<script>bad</script>",
        "/media/a.webm`",
        "javascript:alert(1)",
        "/media/a.webm\ranything",
        "https://youtu.be/not-an-id",
    ] {
        assert!(video_block(source).is_none(), "{source}");
    }
}
