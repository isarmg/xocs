//! References in saved content that retain managed media, including drafts.
use pulldown_cmark::{
    CodeBlockKind, Event, Options, Parser, Tag, TagEnd,
    autolink::{LinkFinder, LinkKind},
};
use sqlx::{Row, SqliteConnection};
use url_encoding::address::Url;

use super::formats::MediaFilename;
use crate::{AppError, db_error};

enum TextContext {
    Prose,
    Code,
    Video(String),
}

fn decoded_reference(value: &str) -> String {
    let mut decoded = url_encoding::percent::percent_decode_str(value)
        .decode_utf8_lossy()
        .into_owned();
    // HTML attributes may use numeric entities; Markdown and ordinary URL
    // fields may percent-encode the same image path. Compare their decoded form.
    for _ in 0..3 {
        let mut output = String::with_capacity(decoded.len());
        let mut rest = decoded.as_str();
        while let Some(start) = rest.find('&') {
            output.push_str(&rest[..start]);
            let candidate = &rest[start..];
            let Some(end) = candidate.find(';').filter(|end| *end <= 16) else {
                output.push('&');
                rest = &candidate[1..];
                continue;
            };
            let entity = &candidate[1..end];
            let character = match entity {
                "sol" => Some('/'),
                "period" => Some('.'),
                "amp" => Some('&'),
                _ => entity
                    .strip_prefix("#x")
                    .or_else(|| entity.strip_prefix("#X"))
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .and_then(char::from_u32)
                    .or_else(|| {
                        entity
                            .strip_prefix('#')
                            .and_then(|number| number.parse().ok())
                            .and_then(char::from_u32)
                    }),
            };
            if let Some(character) = character {
                output.push(character);
                rest = &candidate[end + 1..];
            } else {
                output.push('&');
                rest = &candidate[1..];
            }
        }
        output.push_str(rest);
        output = url_encoding::percent::percent_decode_str(&output)
            .decode_utf8_lossy()
            .into_owned();
        if output == decoded {
            break;
        }
        decoded = output;
    }
    decoded
}

fn text_contains_reference(value: &str, path: &str) -> bool {
    if value.contains(path) {
        return true;
    }
    let decoded = decoded_reference(value);
    if decoded.contains(path) || url_references(value, path) || url_references(&decoded, path) {
        return true;
    }
    // The browser uses this same tokenizer: backslash escapes depend on the
    // Markdown context and must never be stripped from arbitrary stored text.
    // Its destinations have already decoded CommonMark escapes and entities;
    // only URL percent decoding remains, not another HTML-entity pass.
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    let mut context = TextContext::Prose;
    Parser::new_ext(
        value,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .any(|event| match event {
        Event::Start(Tag::CodeBlock(kind)) => {
            context = match kind {
                CodeBlockKind::Fenced(info) if info.trim() == "video" => {
                    TextContext::Video(String::new())
                }
                CodeBlockKind::Indented | CodeBlockKind::Fenced(_) => TextContext::Code,
            };
            false
        }
        Event::End(TagEnd::CodeBlock) => {
            // A video fence is an active media destination. Its text is a URL,
            // not CommonMark, so preserve literal escapes while normalizing the
            // path exactly as for an ordinary media URL field.
            match std::mem::replace(&mut context, TextContext::Prose) {
                TextContext::Video(source) => url_references(source.trim(), path),
                TextContext::Code => false,
                TextContext::Prose => {
                    unreachable!("invariant: CommonMark code block events are balanced")
                }
            }
        }
        Event::Start(Tag::Image { dest_url, .. } | Tag::Link { dest_url, .. }) => {
            dest_url.contains(path)
                || url_encoding::percent::percent_decode_str(&dest_url)
                    .decode_utf8_lossy()
                    .contains(path)
                || url_references(&dest_url, path)
        }
        Event::Text(text) => match &mut context {
            TextContext::Video(video) => {
                video.push_str(&text);
                false
            }
            TextContext::Code => false,
            TextContext::Prose => finder.links(&text).any(|link| {
                if link.kind() != &LinkKind::Url {
                    return false;
                }
                let value = link.as_str();
                if value.split_once("://").is_some_and(|(scheme, _)| {
                    scheme
                        .as_bytes()
                        .first()
                        .is_some_and(u8::is_ascii_alphabetic)
                        && scheme
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"+.-".contains(&byte))
                }) {
                    url_references(value, path)
                } else {
                    url_references(&format!("http://{value}"), path)
                }
            }),
        },
        Event::Start(
            Tag::Paragraph
            | Tag::Heading { .. }
            | Tag::BlockQuote(_)
            | Tag::HtmlBlock
            | Tag::List(_)
            | Tag::Item
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript
            | Tag::MetadataBlock(_),
        )
        | Event::End(
            TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::BlockQuote(_)
            | TagEnd::HtmlBlock
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Table
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell
            | TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::Link
            | TagEnd::Image
            | TagEnd::MetadataBlock(_),
        )
        | Event::Code(_)
        | Event::InlineMath(_)
        | Event::DisplayMath(_)
        | Event::Html(_)
        | Event::InlineHtml(_)
        | Event::FootnoteReference(_)
        | Event::SoftBreak
        | Event::HardBreak
        | Event::Rule
        | Event::TaskListMarker(_) => false,
    })
}

fn url_references(value: &str, path: &str) -> bool {
    let base = Url::parse("https://media.invalid/").expect("static media reference base");
    base.join(value.trim()).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url_encoding::percent::percent_decode_str(url.path()).decode_utf8_lossy() == path
    })
}

fn contains_reference(value: &str, path: &str) -> bool {
    fn json_contains(value: &serde_json::Value, path: &str) -> bool {
        match value {
            serde_json::Value::String(value) => text_contains_reference(value, path),
            serde_json::Value::Array(values) => {
                values.iter().any(|value| json_contains(value, path))
            }
            serde_json::Value::Object(values) => {
                values.values().any(|value| json_contains(value, path))
            }
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
                false
            }
        }
    }
    value.split('\0').any(|field| {
        // Each SQL column is a separate document. Parsing their NUL-separated
        // concatenation changes CommonMark reference-definition boundaries.
        text_contains_reference(field, path)
            || serde_json::from_str::<serde_json::Value>(field)
                .is_ok_and(|value| json_contains(&value, path))
    })
}

pub(super) async fn referenced(
    connection: &mut SqliteConnection,
    filename: &MediaFilename,
) -> Result<bool, AppError> {
    let path = filename.path();
    // Inspect all current content, including drafts, so removing an unused
    // upload cannot break a saved article, profile, album, note or letter.
    const QUERIES: &[&str] = &[
        "SELECT id,coalesce(article_cover,'')||char(0)||coalesce(video_url,'')||char(0)||coalesce(article_content,'')||char(0)||coalesce(tips,'') AS value FROM article NOT INDEXED WHERE deleted=0",
        "SELECT id,coalesce(background_image,'')||char(0)||coalesce(avatar,'')||char(0)||coalesce(random_cover,'')||char(0)||coalesce(notices,'')||char(0)||coalesce(media,'') AS value FROM web_info",
        "SELECT id,coalesce(bg_cover,'')||char(0)||coalesce(man_cover,'')||char(0)||coalesce(woman_cover,'')||char(0)||coalesce(family_info,'') AS value FROM family",
        "SELECT id,coalesce(cover,'')||char(0)||coalesce(url,'')||char(0)||coalesce(introduction,'')||char(0)||coalesce(remark,'') AS value FROM resource_path",
        "SELECT id,coalesce(image_path,'')||char(0)||coalesce(content,'') AS value FROM wei_yan",
        "SELECT id,coalesce(image_path,'')||char(0)||coalesce(avatar,'')||char(0)||coalesce(message,'') AS value FROM tree_hole",
        "SELECT id,coalesce(avatar,'') AS value FROM user WHERE deleted=0",
        "SELECT id,coalesce(comment_content,'')||char(0)||coalesce(comment_info,'') AS value FROM comment",
    ];
    for query in QUERIES {
        // Every browser-normalized alias retains the uploaded filename, unless
        // it is encoded. Match that filename before checking URL destinations;
        // matching only the canonical path would miss /media/./ and ../ aliases.
        // Encoded references are checked in bounded individual rows.
        // The article feed indexes order by date, so using them here would sort
        // the remaining corpus on every batch. Walk its rowid range instead.
        let mut after = i64::MIN;
        loop {
            let rows = sqlx::query(sqlx::AssertSqlSafe(format!("SELECT id,value FROM ({query}) WHERE id>? AND (instr(value,?)>0 OR instr(value,'%')>0 OR instr(value,'&')>0 OR instr(value,char(92))>0) ORDER BY id LIMIT 8")))
                .bind(after).bind(filename.as_str()).fetch_all(&mut *connection).await.map_err(db_error)?;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                after = row.try_get("id").map_err(db_error)?;
                let value: String = row.try_get("value").map_err(db_error)?;
                if contains_reference(&value, &path) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
#[path = "../../../../test/server/unit/media/references.rs"]
mod tests;
