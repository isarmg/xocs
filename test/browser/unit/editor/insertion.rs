use super::{Insertion, UnfinishedFormatting};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

const BLOCK: &str = "```video\n/media/clip.webm\n```\n";

#[test]
fn utf16_selection_replaces_emoji_without_splitting_unicode() {
    let insertion = Insertion::new("甲😀乙尾".into(), Some((1, 3)));
    let (result, cursor) = insertion.apply("甲😀乙尾", BLOCK).unwrap();
    assert_eq!(result, "甲\n\n```video\n/media/clip.webm\n```\n\n乙尾");
    assert_eq!(
        result.encode_utf16().take(cursor as usize).count(),
        cursor as usize
    );
    assert_eq!(
        result.encode_utf16().nth(cursor as usize),
        Some('乙' as u16)
    );
}

#[test]
fn untouched_editor_appends_and_repeated_insertions_keep_blank_lines() {
    let insertion = Insertion::new("intro".into(), None);
    let (first, _) = insertion.apply("intro", BLOCK).unwrap();
    assert_eq!(first, "intro\n\n```video\n/media/clip.webm\n```\n");
    let (second, _) = Insertion::new(first.clone(), None)
        .apply(&first, BLOCK)
        .unwrap();
    assert_eq!(second.matches("```video").count(), 2);
    assert!(second.contains("```\n\n```video"));
}

#[test]
fn uploads_rebase_around_changes_before_and_after_the_selection() {
    let insertion = Insertion::new("head selected tail".into(), Some((5, 13)));
    let (result, _) = insertion.apply("new head selected tail", BLOCK).unwrap();
    assert_eq!(
        result,
        "new head \n\n```video\n/media/clip.webm\n```\n\n tail"
    );
    let (result, _) = insertion
        .apply("head selected tail extended", BLOCK)
        .unwrap();
    assert_eq!(
        result,
        "head \n\n```video\n/media/clip.webm\n```\n\n tail extended"
    );
}

#[test]
fn changes_inside_the_selection_are_preserved_instead_of_overwritten() {
    let insertion = Insertion::new("head selected tail".into(), Some((5, 13)));
    let (result, _) = insertion.apply("head revised tail", BLOCK).unwrap();
    assert!(result.starts_with("head revised\n\n```video"), "{result}");
    assert!(result.ends_with("\n\n tail"), "{result}");
    let insertion = Insertion::new("甲😀乙".into(), Some((3, 3)));
    let (result, _) = insertion.apply("甲😀新内容乙", BLOCK).unwrap();
    assert!(result.starts_with("甲😀新内容\n\n```video"), "{result}");
    assert!(result.ends_with("\n\n乙"), "{result}");
}

#[test]
fn stale_or_invalid_ranges_clamp_to_character_boundaries() {
    let insertion = Insertion::new("甲😀乙".into(), Some((u32::MAX, 2)));
    let (result, _) = insertion.apply("甲😀乙", BLOCK).unwrap();
    assert!(result.starts_with("甲\n\n```video"), "{result}");
    let (result, _) = insertion.apply("", BLOCK).unwrap();
    assert_eq!(result, "```video\n/media/clip.webm\n```\n");
}

fn players(source: &str) -> usize {
    Parser::new_ext(source, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH)
        .filter(|event| {
            matches!(event, Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if info.trim() == "video")
        })
        .count()
}

fn inserted_at(source: &str, start: usize, end: usize) -> Result<String, UnfinishedFormatting> {
    let start = source[..start].encode_utf16().count() as u32;
    let end = source[..end].encode_utf16().count() as u32;
    Insertion::new(source.into(), Some((start, end)))
        .apply(source, BLOCK)
        .map(|(value, _)| value)
}

#[test]
fn code_block_insertions_preserve_code_and_trailing_content() {
    for source in [
        "```rust\nlet keep = 1;\n```\n\nAfter",
        "~~~~rust\nlet keep = 1;\n~~~~\n\nAfter",
        "    let keep = 1;\n    keep();\n\nAfter",
    ] {
        let at = source.find("keep").unwrap() + 2;
        let result = inserted_at(source, at, at + 2).unwrap();
        assert_eq!(players(&result), 1, "{result}");
        let original = source.split("\n\nAfter").next().unwrap();
        assert!(result.starts_with(original), "{result}");
        assert!(result.ends_with("After"), "{result}");
        assert!(
            result.find("```video").unwrap() > original.len(),
            "{result}"
        );
    }
    let source = "    indented code\n\nAfter";
    let result = inserted_at(source, 2, 2).unwrap();
    assert!(
        result.starts_with("    indented code\n\n```video"),
        "{result}"
    );
    assert_eq!(players(&result), 1);
}

#[test]
fn partial_inline_constructs_move_after_the_intact_paragraph() {
    for (source, needle) in [
        ("before **hello** after\n\nTail", "hello"),
        ("before *hello* after\n\nTail", "hello"),
        ("before ~~hello~~ after\n\nTail", "hello"),
        ("before `hello` after\n\nTail", "hello"),
        ("before [hello](https://example.org) after\n\nTail", "hello"),
        (
            "before [hello](https://example.org) after\n\nTail",
            "example",
        ),
        ("before ![hello](/media/image.png) after\n\nTail", "image"),
        (
            "before <span title=\"hello\">inside</span> after\n\nTail",
            "hello",
        ),
        ("before <span>hello</span> after\n\nTail", "hello"),
    ] {
        let at = source.find(needle).unwrap() + 1;
        let result = inserted_at(source, at, at + 1).unwrap();
        let original = source.split("\n\nTail").next().unwrap();
        assert!(result.starts_with(original), "{result}");
        assert!(
            result.find("```video").unwrap() > original.len(),
            "{result}"
        );
        assert_eq!(players(&result), 1, "{result}");
        assert!(result.ends_with("Tail"), "{result}");
    }
}

#[test]
fn containers_and_structured_blocks_remain_whole() {
    for (source, needle) in [
        ("- first\n- second\n\nTail", "first"),
        ("> first\n> second\n\nTail", "first"),
        ("## first\n\nTail", "first"),
        ("first\n=====\n\nTail", "first"),
        ("| first | last |\n|---|---|\n| a | b |\n\nTail", "first"),
        ("<div>\nfirst\n</div>\n\nTail", "first"),
        ("- quoted\n  > first\n  > second\n- last\n\nTail", "first"),
        ("---\n\nTail", "---"),
    ] {
        let at = source.find(needle).unwrap() + 1;
        let result = inserted_at(source, at, at).unwrap();
        let original = source.split("\n\nTail").next().unwrap();
        assert!(result.starts_with(original), "{result}");
        assert!(
            result.find("```video").unwrap() > original.len(),
            "{result}"
        );
        assert_eq!(players(&result), 1, "{result}");
        assert!(result.ends_with("Tail"), "{result}");
    }
}

#[test]
fn link_definitions_are_protected_even_without_events() {
    let source = "[ref]: https://example.org/path\n  \"multi line title\"\n\n[hello][ref]";
    let at = source.find("example").unwrap() + 2;
    let result = inserted_at(source, at, at).unwrap();
    assert!(
        result.starts_with(source.split("\n\n[hello]").next().unwrap()),
        "{result}"
    );
    assert_eq!(players(&result), 1, "{result}");
    assert!(Parser::new(&result).any(|event| matches!(event, Event::Start(Tag::Link { .. }))));
}

#[test]
fn completely_selected_syntax_can_be_replaced() {
    for selected in [
        "**hello**",
        "`hello`",
        "[hello](https://example.org)",
        "![hello](/media/image.png)",
    ] {
        let source = format!("before {selected} after");
        let result = inserted_at(&source, 7, 7 + selected.len()).unwrap();
        assert!(!result.contains(selected), "{result}");
        assert!(result.starts_with("before "), "{result}");
        assert!(result.ends_with(" after"), "{result}");
        assert_eq!(players(&result), 1);
    }
    let source = "```rust\nkeep();\n```";
    let result = inserted_at(source, 0, source.len()).unwrap();
    assert_eq!(result, BLOCK);
}

#[test]
fn unfinished_context_is_rejected_without_rewriting_the_source() {
    for source in [
        "```rust\nlet keep = 1;",
        "~~~~rust\nlet keep = 1;",
        "<script>literal",
        "<!-- comment",
    ] {
        let at = source.find(' ').unwrap_or(source.len() - 1);
        assert_eq!(
            inserted_at(source, at, at),
            Err(UnfinishedFormatting),
            "{source}"
        );
        assert_eq!(
            inserted_at(source, source.len(), source.len()),
            Err(UnfinishedFormatting),
            "{source}"
        );
        // A deliberate insertion before that block is still safe.
        let result = inserted_at(source, 0, 0).unwrap();
        assert!(result.ends_with(source));
        assert_eq!(players(&result), 1);
    }
}

#[test]
fn asynchronous_uploads_recheck_the_latest_markdown_context() {
    let source = "keep();";
    let insertion = Insertion::new(source.into(), Some((3, 3)));
    let current = "```rust\nkeep();\n```\n\nTail";
    let (result, _) = insertion.apply(current, BLOCK).unwrap();
    assert!(
        result.starts_with("```rust\nkeep();\n```\n\nTail"),
        "{result}"
    );
    assert_eq!(players(&result), 1);
    let current = "```rust\nkeep();";
    assert_eq!(insertion.apply(current, BLOCK), Err(UnfinishedFormatting));
}
