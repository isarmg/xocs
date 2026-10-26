//! Preserve edits and Markdown structure when inserting a standalone video block.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct UnfinishedFormatting;

fn parser(source: &str) -> Parser<'_> {
    Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
}

fn cuts(range: &Range<usize>, start: usize, end: usize) -> bool {
    if start == end {
        range.start < start && start < range.end
    } else {
        start < range.end && end > range.start && !(start <= range.start && end >= range.end)
    }
}

fn contextual_range(source: &str, start: usize, end: usize) -> (usize, usize) {
    let parser = parser(source);
    let mut destination = None::<usize>;
    let mut protect = |range: &Range<usize>| {
        if cuts(range, start, end) {
            destination = Some(destination.unwrap_or(end).max(range.end).max(end));
        }
    };
    // Reference definitions do not appear in the event stream. Splitting one
    // would silently change links elsewhere in the article.
    for (_, definition) in parser.reference_definitions().iter() {
        protect(&definition.span);
    }
    let mut root = None::<(Range<usize>, bool)>;
    let mut depth = 0usize;
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    let paragraph = matches!(tag, Tag::Paragraph);
                    let line_start = source[..range.start].rfind('\n').map_or(0, |at| at + 1);
                    let range = line_start..range.end;
                    if !paragraph {
                        protect(&range);
                    }
                    root = Some((range, paragraph));
                } else if let Some((root, true)) = &root
                    && cuts(&range, start, end)
                {
                    // Keep a partially selected inline construct intact and
                    // place the video after its containing paragraph.
                    protect(root);
                }
                depth += 1;
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    root = None;
                }
            }
            Event::Code(_) if cuts(&range, start, end) => {
                if let Some((root, _)) = &root {
                    protect(root);
                }
            }
            Event::InlineHtml(_) => {
                // Offset events identify individual tags, not paired HTML
                // elements. Preserve the whole paragraph around such markup.
                if let Some((root, _)) = &root {
                    protect(root);
                }
            }
            Event::Rule => protect(&range),
            Event::Text(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::TaskListMarker(_) => {}
        }
    }
    destination.map_or((start, end), |at| (at, at))
}

fn standalone_video_at(source: &str, start: usize, end: usize) -> bool {
    let mut depth = 0;
    for (event, range) in parser(source).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == 0
                    && range.start == start
                    && range.end <= end
                    && matches!(tag, Tag::CodeBlock(CodeBlockKind::Fenced(info)) if info.trim() == "video")
                {
                    return true;
                }
                depth += 1;
            }
            Event::End(_) => depth -= 1,
            Event::Text(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule
            | Event::TaskListMarker(_) => {}
        }
    }
    false
}

#[derive(Clone)]
pub(super) struct Insertion {
    source: String,
    start: usize,
    end: usize,
}

fn byte_offset(value: &str, utf16: u32) -> usize {
    let mut units = 0;
    for (offset, ch) in value.char_indices() {
        if units + ch.len_utf16() > utf16 as usize {
            return offset;
        }
        units += ch.len_utf16();
    }
    value.len()
}

impl Insertion {
    pub(super) fn new(source: String, selection: Option<(u32, u32)>) -> Self {
        let (start, end) = selection.map_or((source.len(), source.len()), |(start, end)| {
            let start = byte_offset(&source, start);
            let end = byte_offset(&source, end);
            (start.min(end), start.max(end))
        });
        Self { source, start, end }
    }

    pub(super) fn apply(
        &self,
        current: &str,
        block: &str,
    ) -> Result<(String, u32), UnfinishedFormatting> {
        let (start, end) = self.rebased_range(current);
        let (start, end) = contextual_range(current, start, end);
        let before = &current[..start];
        let after = &current[end..];
        let mut result = String::with_capacity(current.len() + block.len() + 4);
        result.push_str(before);
        if !before.is_empty() && !before.ends_with("\n\n") {
            if !before.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
        }
        let block_start = result.len();
        result.push_str(block.trim_matches('\n'));
        result.push('\n');
        if !after.is_empty() && !after.starts_with('\n') {
            result.push('\n');
        }
        let block_end = result.len();
        let cursor = result.encode_utf16().count().min(u32::MAX as usize) as u32;
        result.push_str(after);
        // An unfinished fence or HTML block can continue past its source range
        // at EOF. Confirm the inserted block really stands on its own rather
        // than closing, entering or changing the author's existing syntax.
        if !standalone_video_at(&result, block_start, block_end) {
            return Err(UnfinishedFormatting);
        }
        Ok((result, cursor))
    }

    fn rebased_range(&self, current: &str) -> (usize, usize) {
        if current == self.source {
            return (self.start, self.end);
        }
        let prefix: usize = self
            .source
            .chars()
            .zip(current.chars())
            .take_while(|(old, new)| old == new)
            .map(|(ch, _)| ch.len_utf8())
            .sum();
        let suffix: usize = self.source[prefix..]
            .chars()
            .rev()
            .zip(current[prefix..].chars().rev())
            .take_while(|(old, new)| old == new)
            .map(|(ch, _)| ch.len_utf8())
            .sum();
        let old_end = self.source.len() - suffix;
        let new_end = current.len() - suffix;
        if self.end <= prefix && self.start < prefix {
            (self.start, self.end)
        } else if self.start >= old_end {
            // The replacement lies after the edited span, including insertions
            // at its old boundary; keep everything the author just typed.
            (new_end + self.start - old_end, new_end + self.end - old_end)
        } else {
            // The author changed the captured selection. Insert after that edit
            // instead of deleting new content using an obsolete selection.
            let anchor = if self.end >= old_end {
                new_end + self.end - old_end
            } else {
                new_end
            };
            (anchor, anchor)
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/browser/unit/editor/insertion.rs"]
mod tests;
