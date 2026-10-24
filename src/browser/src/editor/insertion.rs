//! Rebase a captured textarea selection without discarding edits made during an upload.

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

    pub(super) fn apply(&self, current: &str, block: &str) -> (String, u32) {
        let (start, end) = self.rebased_range(current);
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
        result.push_str(block.trim_matches('\n'));
        result.push('\n');
        if !after.is_empty() && !after.starts_with('\n') {
            result.push('\n');
        }
        let cursor = result.encode_utf16().count().min(u32::MAX as usize) as u32;
        result.push_str(after);
        (result, cursor)
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
