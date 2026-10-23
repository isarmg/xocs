//! Markdown is parsed as structured CommonMark events, never as trusted HTML.
//! Raw HTML events become escaped text; URL destinations are admitted before
//! the upstream HTML renderer escapes text and attributes. No sanitizer accepts
//! arbitrary HTML here, and only tables/strikethrough extensions are enabled.
use pulldown_cmark::autolink::{LinkFinder, LinkKind};
use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag, TagEnd, html};

#[cfg(target_arch = "wasm32")]
fn scheme(value: &str) -> Option<String> {
    web_sys::Url::new_with_base(value, "https://markdown.invalid/")
        .ok()
        .map(|url| url.protocol().trim_end_matches(':').to_owned())
}
#[cfg(not(target_arch = "wasm32"))]
fn scheme(value: &str) -> Option<String> {
    url_encoding::address::Url::options()
        .base_url(Some(
            &url_encoding::address::Url::parse("https://markdown.invalid/").ok()?,
        ))
        .parse(value)
        .ok()
        .map(|url| url.scheme().to_owned())
}
fn allowed_destination(value: &str, image: bool) -> bool {
    scheme(value).is_some_and(|scheme| {
        if image {
            ["http", "https"].contains(&scheme.as_str())
        } else {
            ["http", "https", "mailto", "ftp", "ftps", "tel"].contains(&scheme.as_str())
        }
    })
}

fn has_authority_scheme(value: &str) -> bool {
    // LinkFinder's explicit URL scanner recognizes `scheme://authority`.
    // A colon in a bare host's port, path or query is not a scheme delimiter.
    value.split_once("://").is_some_and(|(scheme, _)| {
        scheme
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
            && scheme
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+.-".contains(&b))
    })
}

pub fn html(content: &str) -> String {
    let parser = Parser::new_ext(
        content,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    );
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    let mut events = Vec::new();
    let mut links = Vec::new();
    let mut images = Vec::new();
    let mut code = false;
    for event in parser {
        match event {
            Event::Html(text) | Event::InlineHtml(text) => events.push(Event::Text(text)),
            Event::SoftBreak => events.push(Event::HardBreak),
            Event::Start(Tag::CodeBlock(kind)) => {
                code = true;
                events.push(Event::Start(Tag::CodeBlock(kind)));
            }
            Event::End(TagEnd::CodeBlock) => {
                code = false;
                events.push(Event::End(TagEnd::CodeBlock));
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let destination = if link_type == LinkType::Email {
                    format!("mailto:{dest_url}")
                } else {
                    dest_url.to_string()
                };
                let allowed = allowed_destination(&destination, false);
                links.push(allowed);
                if allowed {
                    events.push(Event::Start(Tag::Link {
                        link_type,
                        dest_url,
                        title,
                        id,
                    }));
                }
            }
            Event::End(TagEnd::Link) => {
                if links.pop() == Some(true) {
                    events.push(Event::End(TagEnd::Link));
                }
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let allowed = allowed_destination(&dest_url, true);
                images.push(allowed);
                if allowed {
                    events.push(Event::Start(Tag::Image {
                        link_type,
                        dest_url,
                        title,
                        id,
                    }));
                }
            }
            Event::End(TagEnd::Image) => {
                if images.pop() == Some(true) {
                    events.push(Event::End(TagEnd::Image));
                }
            }
            Event::Text(text) if !code && links.is_empty() && images.is_empty() => {
                let mut start = 0;
                for link in finder.links(&text) {
                    let destination = match link.kind() {
                        LinkKind::Email => format!("mailto:{}", link.as_str()),
                        LinkKind::Url if !has_authority_scheme(link.as_str()) => {
                            format!("http://{}", link.as_str())
                        }
                        _ => link.as_str().to_owned(),
                    };
                    if !allowed_destination(&destination, false) {
                        continue;
                    }
                    if start < link.start() {
                        events.push(Event::Text(text[start..link.start()].to_owned().into()));
                    }
                    events.push(Event::Start(Tag::Link {
                        link_type: LinkType::Inline,
                        dest_url: destination.into(),
                        title: CowStr::Borrowed(""),
                        id: CowStr::Borrowed(""),
                    }));
                    events.push(Event::Text(link.as_str().to_owned().into()));
                    events.push(Event::End(TagEnd::Link));
                    start = link.end();
                }
                if start == 0 {
                    events.push(Event::Text(text));
                } else if start < text.len() {
                    events.push(Event::Text(text[start..].to_owned().into()));
                }
            }
            event => events.push(event),
        }
    }
    let mut output = String::with_capacity(content.len());
    html::push_html(&mut output, events.into_iter());
    output
}

#[cfg(test)]
mod tests {
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
}
