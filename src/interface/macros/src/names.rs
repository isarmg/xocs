//! Component symbol names; default identifier boundaries from convert_case.
use unicode_segmentation::UnicodeSegmentation;
fn upper(s: &str) -> bool {
    s.to_uppercase() != s.to_lowercase() && s == s.to_uppercase()
}
fn lower(s: &str) -> bool {
    s.to_uppercase() != s.to_lowercase() && s == s.to_lowercase()
}
fn digit(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}
fn words(s: &str) -> Vec<&str> {
    let (indices, graphemes): (Vec<_>, Vec<_>) = s.grapheme_indices(true).unzip();
    let mut words = Vec::new();
    let mut last = 0;
    for (i, first) in graphemes.iter().enumerate() {
        if matches!(*first, "_" | "-" | " ") {
            words.push(&s[last..indices[i]]);
            last = indices[i] + first.len();
        } else if let Some(second) = graphemes.get(i + 1) {
            let transition = lower(first) && upper(second)
                || (lower(first) || upper(first)) && digit(second)
                || digit(first) && (lower(second) || upper(second))
                || upper(first)
                    && upper(second)
                    && graphemes.get(i + 2).is_some_and(|next| lower(next));
            if transition {
                let end = indices[i + 1];
                words.push(&s[last..end]);
                last = end;
            }
        }
    }
    if !s.is_empty() {
        words.push(&s[last..]);
    }
    words
}
pub(crate) fn snake(s: &str) -> String {
    words(s)
        .iter()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}
pub(crate) fn pascal(s: &str) -> String {
    words(s)
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                })
                .unwrap_or_default()
        })
        .collect()
}
#[path = "../../../../test/interface/macros/unit/names/tests.rs"]
#[cfg(test)]
mod tests;
