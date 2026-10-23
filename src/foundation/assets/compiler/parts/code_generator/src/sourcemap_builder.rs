use std::{borrow::Cow, collections::hash_map::Entry, path::Path};

use crate::typed_index::{IndexVec, define_nonmax_u32_index_type};
use crate::source_location::Span;
use crate::syntax_types::line_terminator::{LS, LS_LAST_2_BYTES, LS_OR_PS_FIRST_BYTE, PS, PS_LAST_2_BYTES};
use rustc_hash::FxHashMap;

/// Number of lines to check with linear search when translating byte position to line index
const LINE_SEARCH_LINEAR_ITERATIONS: usize = 16;

define_nonmax_u32_index_type! {
    /// Index into vec of `ColumnOffsets`
    struct ColumnOffsetsId;
}

/// Line offset tables.
///
/// Used for tracking lines and columns from byte offsets via binary search.
///
/// Code is adapted from [esbuild](https://github.com/evanw/esbuild/blob/cc74e6042a9f573bf58e1e3f165ebda70af4ad3b/internal/js_printer/js_printer.go#L4806-L4808)
///
/// Most lines of source code will not contain Unicode chars, so optimize storage for this common case.
///
/// Each line is represented by a `Line`.
/// Where a line is entirely ASCII, translating byte offset to UTF-16 column is simple,
/// given the byte offset of start of line. A column lookup table isn't needed for that line.
/// In this case, `Line::column_offsets_id` is `None`.
/// For rare lines which do contain Unicode chars, we store column offsets in a `ColumnOffsets` which
/// is stored in a separate `IndexVec`. `Line::column_offsets_id` contains index for that line's `ColumnOffsets`.
/// Storing column offset info which is rarely used in a separate structure keeps `Line` as small as possible.
#[derive(Debug, Default)]
pub struct LineOffsetTables {
    lines: Vec<Line>,
    column_offsets: IndexVec<ColumnOffsetsId, ColumnOffsets>,
}

#[derive(Debug)]
pub struct Line {
    byte_offset_to_start_of_line: u32,
    column_offsets_id: Option<ColumnOffsetsId>,
}

#[derive(Debug)]
pub struct ColumnOffsets {
    byte_offset_to_first: u32,
    columns: Box<[u32]>,
}

pub struct SourcemapBuilder<'a> {
    source_name: String,
    original_source: &'a str,
    names: Vec<&'a str>,
    names_map: FxHashMap<&'a str, u32>,
    tokens: Vec<crate::source_map::Token>,
    last_generated_update: usize,
    last_position: Option<u32>,
    line_offset_tables: LineOffsetTables,
    generated_line: u32,
    generated_column: u32,
    /// Tracks the last accessed line index to optimize sequential lookups in `search_original_line_and_column`.
    /// Most calls to this method access positions in increasing order (e.g., when mapping source tokens linearly),
    /// so we can avoid unnecessary binary searches by advancing linearly from this cached index.
    last_line_lookup: u32,
}

impl<'a> SourcemapBuilder<'a> {
    pub fn new(path: &Path, source_text: &'a str) -> Self {
        let line_offset_tables = Self::generate_line_offset_tables(source_text);
        // Estimate mapping and name counts from source length to reduce reallocations.
        let source_len = source_text.len();
        Self {
            source_name: path.to_string_lossy().into_owned(),
            original_source: source_text,
            names: Vec::with_capacity(source_len / 64),
            names_map: FxHashMap::default(),
            tokens: Vec::with_capacity(source_len / 4),
            last_generated_update: 0,
            last_position: None,
            line_offset_tables,
            generated_line: 0,
            generated_column: 0,
            last_line_lookup: 0,
        }
    }

    pub fn into_sourcemap(mut self) -> crate::source_map::SourceMap<'a> {
        self.names.shrink_to_fit();
        self.tokens.shrink_to_fit();

        crate::source_map::SourceMap::from_parts(crate::source_map::SourceMapParts {
            file: None,
            names: self.names.into_iter().map(Cow::Borrowed).collect(),
            source_root: None,
            sources: vec![Cow::Owned(self.source_name)],
            source_contents: vec![Some(Cow::Borrowed(self.original_source))],
            tokens: self.tokens.into_boxed_slice(),
            token_chunks: None,
            ignore_list: None,
            debug_id: None,
        })
    }

    pub fn add_source_mapping_for_name(&mut self, output: &[u8], span: Span, name: &str) {
        let original_name = self.original_name(span, name);
        // The token name should be original name.
        // If it hasn't changed, name should be `None` to reduce `SourceMap` size.
        let token_name = if original_name == Some(name) { None } else { original_name };
        self.add_source_mapping(output, span.start, token_name);
    }

    /// Add a mapping for a private identifier, which prints as `#` followed by `name`.
    ///
    /// The token is `#name`, so that is what the mapping is compared against and what it records.
    pub fn add_source_mapping_for_private_name(&mut self, output: &[u8], span: Span, name: &str) {
        let original_name = self.original_name(span, name);
        let unchanged = original_name.is_some_and(|original| {
            original
                .as_bytes()
                .split_first()
                .is_some_and(|(&hash, rest)| hash == b'#' && rest == name.as_bytes())
        });
        let token_name = if unchanged { None } else { original_name };
        self.add_source_mapping(output, span.start, token_name);
    }

    /// Get the source text `span` covers, which is the name to record unless nothing was renamed.
    fn original_name(&self, span: Span, name: &str) -> Option<&'a str> {
        debug_assert!(
            (span.end as usize) <= self.original_source.len(),
            "violated {}:{} <= {} for {name}",
            span.start,
            span.end,
            self.original_source.len()
        );
        self.original_source.get(span.start as usize..span.end as usize)
    }

    pub fn add_source_mapping(&mut self, output: &[u8], position: u32, name: Option<&'a str>) {
        if self.last_position == Some(position) {
            return;
        }
        let (original_line, original_column) = self.search_original_line_and_column(position);
        self.update_generated_line_and_column(output);
        let name_id = name.map(|s| self.add_name(s));
        self.tokens.push(crate::source_map::Token::new(
            self.generated_line,
            self.generated_column,
            original_line,
            original_column,
            Some(0),
            name_id,
        ));
        self.last_position = Some(position);
    }

    fn add_name(&mut self, name: &'a str) -> u32 {
        match self.names_map.entry(name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = u32::try_from(self.names.len())
                    .expect("sourcemap names length should fit in u32");
                entry.insert(id);
                self.names.push(name);
                id
            }
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    fn search_original_line_and_column(&mut self, position: u32) -> (u32, u32) {
        let original_line = self.search_original_line(position);

        // Store line index as starting point for next search
        self.last_line_lookup = original_line as u32;

        let line = &self.line_offset_tables.lines[original_line];
        let mut original_column = position - line.byte_offset_to_start_of_line;
        if let Some(column_offsets_id) = line.column_offsets_id {
            let column_offsets = &self.line_offset_tables.column_offsets[column_offsets_id];
            if original_column >= column_offsets.byte_offset_to_first {
                original_column = column_offsets.columns
                    [(original_column - column_offsets.byte_offset_to_first) as usize];
            }
        }
        (original_line as u32, original_column)
    }

    /// Find line index for byte index `position`, using line offset table.
    ///
    /// Usually output code is roughly in same order as it was in original source,
    /// so line will be close to the line found in last call to this function.
    ///
    /// So do fast linear search first over a few lines, and fallback to slower binary search
    /// if this doesn't find the line.
    fn search_original_line(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let idx = self.last_line_lookup as usize;

        if position >= lines[idx].byte_offset_to_start_of_line {
            self.search_original_line_forwards(position)
        } else {
            self.search_original_line_backwards(position)
        }
    }

    /// Find line index for byte index `position`, starting search at `last_line_lookup`,
    /// and working forwards.
    ///
    /// Search forwards, looking for first line which starts *after* `position`.
    /// `position` then must be on the line before that one.
    fn search_original_line_forwards(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let last_idx = self.last_line_lookup as usize;

        let start_idx = last_idx + 1;
        let end_idx = start_idx + LINE_SEARCH_LINEAR_ITERATIONS;

        // We have a fast path for when there are more than `LINE_SEARCH_LINEAR_ITERATIONS` lines
        // to search (common case unless file is very short).
        // Fast path is do linear search on first `LINE_SEARCH_LINEAR_ITERATIONS` lines,
        // then fallback to binary search over remaining lines.
        // If less than `LINE_SEARCH_LINEAR_ITERATIONS` lines to search, just do linear search,
        // but that's slower as number of lines being searched is not constant, so loop cannot be unrolled.
        if end_idx > lines.len() {
            // Less than `LINE_SEARCH_LINEAR_ITERATIONS` lines to search. Take slow path.
            // Unless file is very short, this branch is rarely taken.
            return self.search_original_line_forwards_when_few_lines(position);
        }

        // Linear search for `LINE_SEARCH_LINEAR_ITERATIONS` lines.
        // Compiler should unroll this loop as it has constant number of iterations.
        // https://godbolt.org/z/heh1cnYa4
        for (line_idx, line) in lines[start_idx..end_idx].iter().enumerate() {
            if line.byte_offset_to_start_of_line > position {
                // This line starts after `position`. `position` must be on previous line.
                return start_idx + line_idx - 1;
            }
        }

        // Line not found yet. Binary search over remaining lines.
        lines[end_idx..].partition_point(|line| line.byte_offset_to_start_of_line <= position)
            + end_idx
            - 1
    }

    #[cold]
    fn search_original_line_forwards_when_few_lines(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let last_idx = self.last_line_lookup as usize;
        let start_idx = last_idx + 1;

        for (line_idx, line) in lines[start_idx..].iter().enumerate() {
            if line.byte_offset_to_start_of_line > position {
                // This line starts after `position`. `position` must be on previous line.
                return start_idx + line_idx - 1;
            }
        }

        // No line starts after `position`. `position` must be on last line.
        lines.len() - 1
    }

    fn search_original_line_backwards(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let mut idx = self.last_line_lookup as usize;

        while lines[idx].byte_offset_to_start_of_line > position {
            idx -= 1;
        }

        if lines[idx].byte_offset_to_start_of_line < position {
            idx = lines
                .partition_point(|table| table.byte_offset_to_start_of_line <= position)
                .saturating_sub(1);
        }

        idx
    }

    #[expect(clippy::cast_possible_truncation)]
    fn update_generated_line_and_column(&mut self, output: &[u8]) {
        // Bytes processed per iteration when fast-forwarding over "boring" bytes
        // (ASCII that is neither a line break nor part of a Unicode char).
        const STRIDE: usize = 8;
        const LO: u64 = 0x0101_0101_0101_0101;
        const HI: u64 = 0x8080_8080_8080_8080;

        let len = output.len();
        let start_index = self.last_generated_update;

        // Find last line break
        let mut line_start_index = start_index;
        let mut idx = start_index;
        let mut last_line_is_ascii = true;

        while idx < len {
            // Fast-forward over runs of "boring" bytes `STRIDE` at a time. The vast
            // majority of output is ASCII that is neither a line break nor part of a
            // Unicode char; such bytes don't change the line count or the last-line
            // ASCII flag, so a run of them can be skipped in bulk instead of matched
            // byte-by-byte. SWAR (SIMD-within-a-register) detects, branchlessly, whether
            // any byte of an 8-byte word is `\n` (0x0A), `\r` (0x0D), or non-ASCII
            // (>= 0x80) using the classic "zero byte in a word" bit trick; if none are,
            // the whole word is skipped.
            while idx + STRIDE <= len {
                let word = u64::from_ne_bytes(output[idx..idx + STRIDE].try_into().unwrap());
                let lf = word ^ (LO * u64::from(b'\n'));
                let cr = word ^ (LO * u64::from(b'\r'));
                let has_lf = lf.wrapping_sub(LO) & !lf & HI;
                let has_cr = cr.wrapping_sub(LO) & !cr & HI;
                let non_ascii = word & HI;
                if (has_lf | has_cr | non_ascii) != 0 {
                    break;
                }
                idx += STRIDE;
            }
            if idx >= len {
                break;
            }

            let byte = output[idx];
            match byte {
                b'\n' => {}
                b'\r' => {
                    // Handle Windows-specific "\r\n" newlines
                    if output.get(idx + 1) == Some(&b'\n') {
                        idx += 1;
                    }
                }
                _ if byte.is_ascii() => {
                    idx += 1;
                    continue;
                }
                LS_OR_PS_FIRST_BYTE => {
                    let next_two_bytes = output.get(idx + 1..idx + 3);
                    if next_two_bytes != Some(&LS_LAST_2_BYTES[..])
                        && next_two_bytes != Some(&PS_LAST_2_BYTES[..])
                    {
                        last_line_is_ascii = false;
                        // 3-byte Unicode char that isn't a line break.
                        // We know it's 3 bytes, so we could do `idx += 3`, but that's more instruction bytes
                        // than `idx += 1` (`inc` instruction). This branch is extremely rarely taken, and this
                        // is in a hot loop, so it's better to optimize for the common case.
                        // The remaining 2 bytes will hit the "Unicode char" branch below on next turns of the loop,
                        // and they'll be skipped too.
                        idx += 1;
                        continue;
                    }
                    // 3-byte line break. Add 2 to `idx` here, as 1 more is added below.
                    idx += 2;
                }
                _ => {
                    // Unicode char
                    last_line_is_ascii = false;
                    // Note: Only increment `idx` by 1. We don't know how many bytes the char is, and non-ASCII
                    // chars are rare, so it's not worthwhile adding more instructions to this hot loop to find out.
                    // The remaining bytes of the char will hit this branch again on next turns of the loop,
                    // and they'll be skipped too.
                    idx += 1;
                    continue;
                }
            }

            // Line break found.
            line_start_index = idx + 1;
            self.generated_line += 1;
            self.generated_column = 0;
            last_line_is_ascii = true;
            idx += 1;
        }

        // Calculate column
        self.generated_column += if last_line_is_ascii {
            (len - line_start_index) as u32
        } else {
            // TODO: It'd be better if could use `from_utf8_unchecked` here, but we'd need to make this
            // function unsafe and caller guarantees `output` contains a valid UTF-8 string
            let last_line = std::str::from_utf8(&output[line_start_index..]).unwrap();
            // Mozilla's "source-map" library counts columns using UTF-16 code units
            last_line.encode_utf16().count() as u32
        };
        self.last_generated_update = len;
    }

    fn generate_line_offset_tables(content: &str) -> LineOffsetTables {
        let mut lines = Vec::with_capacity(content.len() / 24 + 1);
        let mut column_offsets = IndexVec::new();

        // Used as a buffer to reduce memory reallocations.
        // Pre-allocate with reasonable capacity to avoid frequent reallocations.
        // 16 is a guess, probably adequate for most files which don't heavily use unicode chars.
        // 16 entries is 64 bytes = 1 CPU cache line on most CPUs.
        // If file does heavily use unicode chars, `columns` will grow adaptively as needed.
        let mut columns = Vec::with_capacity(16);

        // Process content line-by-line.
        // For each line, start by assuming line will be entirely ASCII, and read byte-by-byte.
        // If line is all ASCII, UTF-8 columns and UTF-16 columns are the same,
        // so no need to create a `columns` Vec. This is the fast path for common case.
        // If a Unicode character found, read rest of line char-by-char, populating `columns` Vec.
        // At end of line, go back to top of outer loop, and again assume ASCII for next line.
        let mut line_byte_offset = 0;
        'lines: loop {
            lines.push(Line {
                byte_offset_to_start_of_line: line_byte_offset,
                column_offsets_id: None,
            });

            let remaining = &content.as_bytes()[line_byte_offset as usize..];
            for (byte_offset_from_line_start, b) in remaining.iter().enumerate() {
                #[expect(clippy::cast_possible_truncation)]
                let mut byte_offset_from_line_start = byte_offset_from_line_start as u32;
                match b {
                    b'\n' => {
                        byte_offset_from_line_start += 1;
                    }
                    b'\r' => {
                        byte_offset_from_line_start += 1;
                        // Handle Windows-specific "\r\n" newlines
                        if remaining.get(byte_offset_from_line_start as usize) == Some(&b'\n') {
                            byte_offset_from_line_start += 1;
                        }
                    }
                    _ if b.is_ascii() => {
                        continue;
                    }
                    _ => {
                        // Unicode char found.
                        // Set `column_offsets_id` for line and create `columns` Vec.
                        let line = lines.iter_mut().last().unwrap();
                        line.column_offsets_id =
                            Some(ColumnOffsetsId::from_usize(column_offsets.len()));

                        // Loop through rest of line char-by-char.
                        // `chunk_byte_offset` in this loop is byte offset from start of this 1st
                        // Unicode char.
                        let mut column = byte_offset_from_line_start;
                        line_byte_offset += byte_offset_from_line_start;
                        let remaining = &content[line_byte_offset as usize..];
                        for (chunk_byte_offset, ch) in remaining.char_indices() {
                            #[expect(clippy::cast_possible_truncation)]
                            let mut chunk_byte_offset = chunk_byte_offset as u32;
                            columns.extend(std::iter::repeat_n(column, ch.len_utf8()));

                            match ch {
                                '\r' => {
                                    // Handle Windows-specific "\r\n" newlines
                                    chunk_byte_offset += 1;
                                    if remaining.as_bytes().get(chunk_byte_offset as usize)
                                        == Some(&b'\n')
                                    {
                                        chunk_byte_offset += 1;
                                        columns.push(column + 1);
                                    }
                                }
                                '\n' => {
                                    chunk_byte_offset += 1;
                                }
                                LS | PS => {
                                    chunk_byte_offset += 3;
                                }
                                #[expect(clippy::cast_possible_truncation)]
                                _ => {
                                    // Mozilla's "source-map" library counts columns using UTF-16 code units
                                    column += ch.len_utf16() as u32;
                                    continue;
                                }
                            }

                            // Line break found.
                            // `chunk_byte_offset` is now the offset of *end* of the line break.
                            line_byte_offset += chunk_byte_offset;

                            // Record column offsets.
                            // `columns.clone().into_boxed_slice()` does perform an allocation,
                            // but only one - `Vec::clone` produces a `Vec` with `capacity == len`,
                            // so `Vec::into_boxed_slice` just drops the `capacity` field,
                            // and does not need to reallocate again.
                            // `columns` is reused for next line, and will grow adaptively depending on
                            // how heavily the file uses unicode chars.
                            column_offsets.push(ColumnOffsets {
                                byte_offset_to_first: byte_offset_from_line_start,
                                columns: columns.clone().into_boxed_slice(),
                            });
                            columns.clear();

                            // Revert back to outer loop for next line
                            continue 'lines;
                        }

                        // EOF.
                        // One last column entry for EOF position.
                        columns.push(column);

                        // Record column offsets
                        column_offsets.push(ColumnOffsets {
                            byte_offset_to_first: byte_offset_from_line_start,
                            columns: columns.into_boxed_slice(),
                        });

                        break 'lines;
                    }
                }

                // Line break found.
                // `byte_offset_from_line_start` is now the length of line *including* line break.
                line_byte_offset += byte_offset_from_line_start;
                continue 'lines;
            }

            // EOF
            break;
        }

        LineOffsetTables { lines, column_offsets }
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/code_generator/src/sourcemap_builder/test.rs"]
#[cfg(test)]
mod test;
