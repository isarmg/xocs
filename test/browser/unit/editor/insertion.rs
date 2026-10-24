use super::Insertion;

const BLOCK: &str = "```video\n/media/clip.webm\n```\n";

#[test]
fn utf16_selection_replaces_emoji_without_splitting_unicode() {
    let insertion = Insertion::new("甲😀乙尾".into(), Some((1, 3)));
    let (result, cursor) = insertion.apply("甲😀乙尾", BLOCK);
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
    let (first, _) = insertion.apply("intro", BLOCK);
    assert_eq!(first, "intro\n\n```video\n/media/clip.webm\n```\n");
    let (second, _) = Insertion::new(first.clone(), None).apply(&first, BLOCK);
    assert_eq!(second.matches("```video").count(), 2);
    assert!(second.contains("```\n\n```video"));
}

#[test]
fn uploads_rebase_around_changes_before_and_after_the_selection() {
    let insertion = Insertion::new("head selected tail".into(), Some((5, 13)));
    let (result, _) = insertion.apply("new head selected tail", BLOCK);
    assert_eq!(
        result,
        "new head \n\n```video\n/media/clip.webm\n```\n\n tail"
    );
    let (result, _) = insertion.apply("head selected tail extended", BLOCK);
    assert_eq!(
        result,
        "head \n\n```video\n/media/clip.webm\n```\n\n tail extended"
    );
}

#[test]
fn changes_inside_the_selection_are_preserved_instead_of_overwritten() {
    let insertion = Insertion::new("head selected tail".into(), Some((5, 13)));
    let (result, _) = insertion.apply("head revised tail", BLOCK);
    assert!(result.starts_with("head revised\n\n```video"), "{result}");
    assert!(result.ends_with("\n\n tail"), "{result}");
    let insertion = Insertion::new("甲😀乙".into(), Some((3, 3)));
    let (result, _) = insertion.apply("甲😀新内容乙", BLOCK);
    assert!(result.starts_with("甲😀新内容\n\n```video"), "{result}");
    assert!(result.ends_with("\n\n乙"), "{result}");
}

#[test]
fn stale_or_invalid_ranges_clamp_to_character_boundaries() {
    let insertion = Insertion::new("甲😀乙".into(), Some((u32::MAX, 2)));
    let (result, _) = insertion.apply("甲😀乙", BLOCK);
    assert!(result.starts_with("甲\n\n```video"), "{result}");
    let (result, _) = insertion.apply("", BLOCK);
    assert_eq!(result, "```video\n/media/clip.webm\n```\n");
}
