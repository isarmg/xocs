//! Rope

pub use ropey::Rope;

/// Get UTF16 line and column from UTF8 offset and source text.
#[expect(clippy::cast_possible_truncation)]
pub fn get_line_column(rope: &Rope, offset: u32, source_text: &str) -> (u32, u32) {
    let offset = offset as usize;
    // Get line number and byte offset of start of line
    let line_index = rope.byte_to_line(offset);
    let line_offset = rope.line_to_byte(line_index);
    // Get column number
    let column_index = source_text[line_offset..offset].encode_utf16().count();
    (line_index as u32, column_index as u32)
}

/// Get UTF8 offset from UTF16 line and column
#[expect(clippy::cast_possible_truncation)]
pub fn get_offset_from_line_and_column(rope: &Rope, line: u32, column: u32) -> u32 {
    let line = line as usize;
    let column = column as usize;
    let line_offset = rope.line_to_byte(line);

    let line_text = rope.line(line);
    let mut utf16_count = 0;
    let mut byte_count = 0;
    for ch in line_text.chars() {
        utf16_count += ch.len_utf16();
        // If adding this character would exceed the target column, return current byte offset
        if utf16_count > column {
            break;
        }
        byte_count += ch.len_utf8();
    }
    // If column equals the line length, return end-of-line byte offset
    (line_offset + byte_count) as u32
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/rope/test.rs"]
#[cfg(test)]
mod test;
