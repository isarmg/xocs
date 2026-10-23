//! Ported and modified from <https://github.com/getsentry/rust-sourcemap/blob/9.1.0/src/encoder.rs>

use std::ops::{Deref, DerefMut};

use crate::json_escaping::escape_into;

use crate::source_map::{JSONSourceMap, SourceMap, Token, token::TokenChunk};

pub fn encode(sourcemap: &SourceMap<'_>) -> JSONSourceMap {
    JSONSourceMap {
        version: 3,
        file: sourcemap.get_file().map(ToString::to_string),
        mappings: {
            let mut mappings = String::with_capacity(estimate_mappings_length(sourcemap));
            serialize_sourcemap_mappings(sourcemap, &mut mappings);
            mappings
        },
        source_root: sourcemap.get_source_root().map(ToString::to_string),
        sources: sourcemap.sources.iter().map(ToString::to_string).collect(),
        sources_content: sourcemap.source_contents.iter().any(Option::is_some).then(|| {
            sourcemap.get_source_contents().map(|content| content.map(str::to_owned)).collect()
        }),
        names: sourcemap.names.iter().map(ToString::to_string).collect(),
        debug_id: sourcemap.get_debug_id().map(ToString::to_string),
        ignore_list: sourcemap.ignore_list.clone(),
    }
}

pub fn encode_to_string(sourcemap: &SourceMap<'_>) -> String {
    // Reserve for worst-case string escaping and an estimate of the mappings.
    // Large VLQ deltas can exceed the estimate; serialization reserves more as needed.
    let mut estimated_capacity = 0usize;

    // {"version":3,
    estimated_capacity += 13;

    // Optional "file":"...",
    if let Some(file) = sourcemap.get_file() {
        estimated_capacity += 8 /* "file": */ + file.len() * 6 + 2 /* quotes */ + 1 /* , */;
    }

    // Optional "sourceRoot":"...",
    if let Some(source_root) = sourcemap.get_source_root() {
        estimated_capacity += 14 /* "sourceRoot": */ + source_root.len() * 6 + 2 /* quotes */ + 1 /* , */;
    }

    // Calculate string lengths in a single pass for better cache locality
    let names_count = sourcemap.names.len();
    let sources_count = sourcemap.sources.len();
    // Accumulate total string bytes across all collections
    let mut total_string_bytes = 0usize;

    for name in &sourcemap.names {
        total_string_bytes += name.len();
    }

    for source in &sourcemap.sources {
        total_string_bytes += source.len();
    }

    // Single pass over source_contents to check existence and accumulate byte lengths
    let (has_source_contents, sc_bytes) =
        sourcemap.source_contents.iter().fold((false, 0usize), |(has_some, bytes), content| {
            match content {
                Some(s) => (true, bytes + s.len()),
                None => (has_some, bytes + 4), // "null"
            }
        });
    total_string_bytes += sc_bytes;
    let sc_count = if has_source_contents { sourcemap.source_contents.len() } else { 0 };

    // Calculate total capacity needed
    estimated_capacity += 9 + 13; // "names":[ + ],"sources":[
    if has_source_contents {
        estimated_capacity += 20; // ],"sourcesContent":[
    }
    estimated_capacity += 6 * total_string_bytes; // worst-case escaping (* 6), \0 -> \\u0000
    estimated_capacity += 2 * (names_count + sources_count + sc_count); // quotes around each item

    // Commas between array items
    let comma_count = names_count.saturating_sub(1)
        + sources_count.saturating_sub(1)
        + sc_count.saturating_sub(1);
    estimated_capacity += comma_count;

    // Optional ],"ignoreList":[
    if let Some(ignore_list) = &sourcemap.ignore_list {
        estimated_capacity += 16; // ],"ignoreList":[

        let ig_count = ignore_list.len();
        // At most 10 digits per u32, plus commas between items.
        estimated_capacity += 10 * ig_count + ig_count.saturating_sub(1);
    }

    // ],"mappings":"
    estimated_capacity += 14;
    estimated_capacity += estimate_mappings_length(sourcemap);

    // Optional ,"debugId":<escaped>
    if let Some(debug_id) = sourcemap.get_debug_id() {
        estimated_capacity += 12 /* ,"debugId": */ + debug_id.len() * 6 + 2 /* quotes */;
    }

    // "} (closing quote of mappings + closing brace)
    estimated_capacity += 2;
    let mut contents = PreAllocatedString::new(estimated_capacity);

    contents.push("{\"version\":3,");
    if let Some(file) = sourcemap.get_file() {
        contents.push("\"file\":");
        escape_into(file, contents.as_mut_vec());
        contents.push(",");
    }

    if let Some(source_root) = sourcemap.get_source_root() {
        contents.push("\"sourceRoot\":");
        escape_into(source_root, contents.as_mut_vec());
        contents.push(",");
    }

    contents.push("\"names\":[");
    contents.push_list(sourcemap.names.iter(), |s, out| escape_into(&**s, out));

    contents.push("],\"sources\":[");
    contents.push_list(sourcemap.sources.iter(), |s, out| escape_into(&**s, out));

    if has_source_contents {
        let source_contents = &sourcemap.source_contents;
        contents.push("],\"sourcesContent\":[");
        contents.push_list(source_contents.iter(), |v, output| match v {
            Some(s) => escape_into(&**s, output),
            None => output.extend_from_slice(b"null"),
        });
    }

    if let Some(ignore_list) = &sourcemap.ignore_list {
        contents.push("],\"ignoreList\":[");
        contents.push_list(ignore_list.iter(), |s, output| {
            output.extend_from_slice(s.to_string().as_bytes());
        });
    }

    contents.push("],\"mappings\":\"");
    serialize_sourcemap_mappings(sourcemap, &mut contents);
    contents.push("\"");

    if let Some(debug_id) = sourcemap.get_debug_id() {
        contents.push(",\"debugId\":");
        escape_into(debug_id, contents.as_mut_vec());
    }

    contents.push("}");

    contents.consume()
}

fn estimate_mappings_length(sourcemap: &SourceMap<'_>) -> usize {
    // Twelve bytes per token is a heuristic, not an upper bound. Generated line
    // breaks contribute once across the whole map, regardless of chunk boundaries.
    sourcemap.tokens.len() * 12 + sourcemap.tokens.last().map_or(0, |t| t.dst_line as usize)
}

fn serialize_sourcemap_mappings(sm: &SourceMap<'_>, output: &mut String) {
    if let Some(token_chunks) = sm.token_chunks.as_ref() {
        token_chunks.iter().for_each(|token_chunk| {
            serialize_mappings(&sm.tokens, token_chunk, output);
        })
    } else {
        serialize_mappings(
            &sm.tokens,
            &TokenChunk::new(0, sm.tokens.len() as u32, 0, 0, 0, 0, 0, 0),
            output,
        );
    }
}

// Max length of a single VLQ encoding
const MAX_VLQ_BYTES: usize = 7;

fn serialize_mappings(tokens: &[Token], token_chunk: &TokenChunk, output: &mut String) {
    let TokenChunk {
        start,
        end,
        mut prev_dst_line,
        mut prev_dst_col,
        mut prev_src_line,
        mut prev_src_col,
        mut prev_name_id,
        mut prev_source_id,
    } = *token_chunk;

    let mut need_comma = start != 0;

    for token in &tokens[start as usize..end as usize] {
        // Max length of a single VLQ encoding is 7 bytes. Max number of calls to `encode_vlq` is 5.
        // Also need 1 byte for each line number difference, or 1 byte if no line num difference.
        // Reserve this amount of capacity in `rv` early, so can skip bounds checks in code below.
        // As well as skipping the bounds checks, this also removes a function call to
        // `alloc::raw_vec::RawVec::grow_one` for every byte that's pushed.
        // https://godbolt.org/z/44G8jjss3
        const MAX_TOTAL_VLQ_BYTES: usize = 5 * MAX_VLQ_BYTES;

        let num_line_breaks = token.get_dst_line() - prev_dst_line;
        if num_line_breaks != 0 {
            let required = MAX_TOTAL_VLQ_BYTES + num_line_breaks as usize;
            if output.capacity() - output.len() < required {
                output.reserve(required);
            }
            // SAFETY: We have reserved sufficient capacity for `num_line_breaks` bytes
            unsafe { push_bytes_unchecked(output, b';', num_line_breaks) };
            prev_dst_col = 0;
            prev_dst_line += num_line_breaks;
        } else if need_comma {
            let required = MAX_TOTAL_VLQ_BYTES + 1;
            if output.capacity() - output.len() < required {
                output.reserve(required);
            }
            // SAFETY: We have reserved sufficient capacity for 1 byte
            unsafe { push_byte_unchecked(output, b',') };
        } else {
            // First token of the first chunk: no delimiter is emitted, but the
            // five VLQ fields below still write up to 35 bytes, and the
            // caller's estimate can be tighter than that (~12 bytes/token), so
            // capacity must still be ensured here.
            if output.capacity() - output.len() < MAX_TOTAL_VLQ_BYTES {
                output.reserve(MAX_TOTAL_VLQ_BYTES);
            }
        }

        if let Some(source_id) = token.get_source_id() {
            // The dominant segment shape in real bundler output is 4 or 5
            // single-digit VLQ fields (small deltas). Transform all field
            // deltas up front so that shape can be detected with one test and
            // written with a single 8-byte store, instead of running the
            // digit-at-a-time VLQ loop per field.
            let v0 = vlq_value(token.get_dst_col(), prev_dst_col);
            let v1 = vlq_value(source_id, prev_source_id);
            let v2 = vlq_value(token.get_src_line(), prev_src_line);
            let v3 = vlq_value(token.get_src_col(), prev_src_col);
            prev_dst_col = token.get_dst_col();
            prev_source_id = source_id;
            prev_src_line = token.get_src_line();
            prev_src_col = token.get_src_col();

            if let Some(name_id) = token.get_name_id() {
                let v4 = vlq_value(name_id, prev_name_id);
                prev_name_id = name_id;
                if !try_push_fast_4_or_5_segment(output, [v0, v1, v2, v3, v4]) {
                    // SAFETY: `MAX_TOTAL_VLQ_BYTES` (35) spare capacity was
                    // ensured above — five `encode_vlq` calls need 7 bytes each.
                    unsafe {
                        encode_vlq(output, v0);
                        encode_vlq(output, v1);
                        encode_vlq(output, v2);
                        encode_vlq(output, v3);
                        encode_vlq(output, v4);
                    }
                }
            } else if !try_push_fast_4_or_5_segment(output, [v0, v1, v2, v3]) {
                // SAFETY: same as above, with only four fields.
                unsafe {
                    encode_vlq(output, v0);
                    encode_vlq(output, v1);
                    encode_vlq(output, v2);
                    encode_vlq(output, v3);
                }
            }
        } else {
            // SAFETY: `MAX_TOTAL_VLQ_BYTES` spare capacity was ensured above.
            unsafe { encode_vlq(output, vlq_value(token.get_dst_col(), prev_dst_col)) };
            prev_dst_col = token.get_dst_col();
        }

        need_comma = true;
    }
}

/// Transform the diff `a - b` into its VLQ integer representation: the sign
/// goes in the low bit and the remaining bits are the magnitude (the
/// branchless inverse of `decode_sign` in `decode.rs`).
/// The result encodes to a single base64 char iff it fits the 5 payload bits
/// of one char, i.e. no bit at or above the continuation bit is set
/// (`value & !0x1F == 0`).
#[inline]
fn vlq_value(a: u32, b: u32) -> u64 {
    (u64::from(a.abs_diff(b)) << 1) | u64::from(a < b)
}

/// Fast path for the dominant sourcemap segment shapes: a 4- or 5-field
/// segment whose VLQ values are all single-digit (no continuation bit).
/// Pushes all fields' base64 chars with a single unaligned 8-byte store and
/// one length update, and returns `true`. Returns `false` without writing
/// anything when any field needs more than one char (or, in theory, when
/// `out` lacks 8 bytes of spare capacity — never the case in
/// `serialize_mappings`, which reserves 35 bytes per token), so the caller
/// can fall back to the per-digit `encode_vlq` loop.
#[inline]
fn try_push_fast_4_or_5_segment<const N: usize>(out: &mut String, vals: [u64; N]) -> bool {
    const { assert!(N <= 8) };

    let mut all = 0u64;
    for &v in &vals {
        all |= v;
    }
    // A value with any bit at or above the VLQ continuation bit needs more
    // than one base64 char. This is the u64-wide analogue of the `& 0xE0`
    // test in `decode.rs` — `!0x1F` rather than `0xE0` because these are
    // full VLQ integers, not single decoded base64 bytes.
    if all & !0x1F != 0 {
        return false;
    }

    // The store below always writes 8 bytes (only the first `N` become
    // visible via `set_len`). The caller reserves more than this for the
    // fallback path anyway, so this predictable branch never fails.
    if out.capacity() - out.len() < 8 {
        return false;
    }

    let mut packed = 0u64;
    for (i, &v) in vals.iter().enumerate() {
        // `& 0x1F` keeps the index provably in bounds for the 64-entry table;
        // it is a no-op here since the mask test above established `v < 32`.
        packed |= u64::from(B64_CHARS.0[(v & 0x1F) as usize]) << (i * 8);
    }

    // SAFETY: the capacity check above guarantees 8 spare bytes for the
    // 8-byte store. All bytes in `B64_CHARS` are ASCII, so the buffer stays
    // valid UTF-8.
    unsafe {
        let vec = out.as_mut_vec();
        let len = vec.len();
        let ptr = vec.as_mut_ptr().add(len);
        // `to_le` so the first char lands at the lowest address on any target.
        ptr.cast::<u64>().write_unaligned(packed.to_le());
        vec.set_len(len + N);
    }
    true
}

// Align chars lookup table on 64 so occupies a single cache line
#[repr(align(64))]
struct Aligned64([u8; 64]);

static B64_CHARS: Aligned64 =
    Aligned64(*b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");

/// Encode a VLQ integer representation (see [`vlq_value`]) and push encoding
/// into `out`. Will push between 1 byte (num = 0) and 7 bytes (num from a
/// diff of -u32::MAX).
///
/// # SAFETY
/// Caller must ensure at least 7 bytes spare capacity in `out`,
/// as this function does not perform any bounds checks.
unsafe fn encode_vlq(out: &mut String, mut num: u64) {
    unsafe {
        // Breaking out of loop early when have reached last char (rather than conditionally adding
        // 32 for last char within the loop) removes 3 instructions from the loop.
        // https://godbolt.org/z/Es4Pavh9j
        // This translates to a 16% speed-up for VLQ encoding.
        // (A packed-u64 variant with a single 8-byte store and one length
        // update was benchmarked at ~25% slower on serialize: it adds ALU work
        // to the dominant single-byte case, and the per-byte length update
        // does not stall in practice.)
        let mut digit;
        loop {
            digit = num & 0b11111;
            num >>= 5;
            if num == 0 {
                break;
            }

            let b = B64_CHARS.0[digit as usize + 32];
            // SAFETY:
            // * This loop can execute a maximum of 7 times, and on last turn will exit before getting here.
            //   Caller promises there are at least 7 bytes spare capacity in `out` at start. We only
            //   push 1 byte on each turn, so guaranteed there is at least 1 byte capacity in `out` here.
            // * All values in `B64_CHARS` lookup table are ASCII bytes.
            push_byte_unchecked(out, b);
        }

        let b = B64_CHARS.0[digit as usize];
        // SAFETY:
        // * The loop above pushes max 6 bytes. Caller promises there are at least 7 bytes spare capacity
        //   in `out` at start. So guaranteed there is at least 1 byte capacity in `out` here.
        // * All values in `B64_CHARS` lookup table are ASCII bytes.
        push_byte_unchecked(out, b);
    }
}

/// Push a byte to `out` without bounds checking.
///
/// # SAFETY
/// * `out` must have at least 1 byte spare capacity.
/// * `b` must be an ASCII byte (i.e. not `>= 128`).
//
// `#[inline(always)]` to ensure that `len` is stored in a register during `encode_vlq`'s loop.
#[expect(clippy::inline_always)]
#[inline(always)]
unsafe fn push_byte_unchecked(out: &mut String, b: u8) {
    unsafe {
        debug_assert!(out.len() < out.capacity());
        debug_assert!(b.is_ascii());

        let out = out.as_mut_vec();
        let len = out.len();
        let ptr = out.as_mut_ptr().add(len);
        ptr.write(b);
        out.set_len(len + 1);
    }
}

/// Push a byte to `out` a number of times without bounds checking.
///
/// # SAFETY
/// * `out` must have at least `repeats` bytes spare capacity.
/// * `b` must be an ASCII byte (i.e. not `>= 128`).
#[inline]
unsafe fn push_bytes_unchecked(out: &mut String, b: u8, repeats: u32) {
    unsafe {
        debug_assert!(out.capacity() - out.len() >= repeats as usize);
        debug_assert!(b.is_ascii());

        let out = out.as_mut_vec();
        let len = out.len();
        let mut ptr = out.as_mut_ptr().add(len);
        for _ in 0..repeats {
            ptr.write(b);
            ptr = ptr.add(1);
        }
        out.set_len(len + repeats as usize);
    }
}

/// A preallocated JSON string buffer that grows as needed.
#[repr(transparent)]
struct PreAllocatedString(String);

impl Deref for PreAllocatedString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PreAllocatedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PreAllocatedString {
    fn new(estimated_capacity: usize) -> Self {
        Self(String::with_capacity(estimated_capacity))
    }

    #[inline]
    fn push(&mut self, s: &str) {
        self.0.push_str(s);
    }

    #[inline]
    fn push_list<S, I>(&mut self, mut iter: I, encode: impl Fn(S, &mut Vec<u8>))
    where
        I: Iterator<Item = S>,
    {
        let Some(first) = iter.next() else {
            return;
        };
        encode(first, self.as_mut_vec());

        for other in iter {
            self.0.push(',');
            encode(other, self.as_mut_vec());
        }
    }

    fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        // SAFETY: we are sure that the string is not shared
        unsafe { self.0.as_mut_vec() }
    }

    #[inline]
    fn consume(self) -> String {
        self.0
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/source_map/src/encode/tests.rs"]
#[cfg(test)]
mod tests;
