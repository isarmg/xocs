/// Port from https://github.com/getsentry/rust-sourcemap/blob/9.1.0/src/decoder.rs
/// It is a helper for decoding VLQ sourcemap strings to `SourceMap`.
use std::borrow::Cow;

#[cfg(any())]
use napi_derive::napi;

use crate::source_map::error::{Error, Result};
use crate::source_map::token::INVALID_ID;
use crate::source_map::{SourceMap, Token};

/// See <https://tc39.es/ecma426/#sec-source-map-format>.
#[cfg_attr(any(), napi(object, js_name = "JSONSourceMap"))]
pub struct JSONSourceMap {
    /// The version field, must be 3.
    pub version: u32,
    /// An optional name of the generated code that this source map is associated with.
    pub file: Option<String>,
    /// A string with the encoded mapping data.
    pub mappings: String,
    /// An optional source root, useful for relocating source files on a server or removing repeated values in the "sources" entry.
    /// This value is prepended to the individual entries in the "source" field.
    pub source_root: Option<String>,
    /// A list of original sources used by the "mappings" entry.
    pub sources: Vec<String>,
    /// An optional list of source content, useful when the "source" can't be hosted.
    /// The contents are listed in the same order as the sources in line 5. "null" may be used if some original sources should be retrieved by name.
    pub sources_content: Option<Vec<Option<String>>>,
    /// A list of symbol names used by the "mappings" entry.
    pub names: Vec<String>,
    /// An optional field containing the debugId for this sourcemap.
    pub debug_id: Option<String>,
    /// Identifies third-party sources (such as framework code or bundler-generated code), allowing developers to avoid code that they don't want to see or step through, without having to configure this beforehand.
    /// The `ignoreList` field refers to the `sources` array, and lists the indices of all the known third-party sources in that source map.
    /// When parsing the source map, developer tools can use this to determine sections of the code that the browser loads and runs that could be automatically ignore-listed.
    /// JSON input falls back to the deprecated `x_google_ignoreList` when `ignoreList` is missing or null.
    #[cfg_attr(any(), napi(js_name = "ignoreList"))]
    pub ignore_list: Option<Vec<u32>>,
}

impl<'de> serde::Deserialize<'de> for JSONSourceMap {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let json: BorrowedJSONSourceMap<'de> = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self {
            version: json.version,
            file: json.file.map(Cow::into_owned),
            mappings: json.mappings.into_owned(),
            source_root: json.source_root.map(Cow::into_owned),
            sources: json.sources.into_iter().map(Cow::into_owned).collect(),
            sources_content: json
                .sources_content
                .map(|content| content.into_iter().map(|item| item.map(Cow::into_owned)).collect()),
            names: json.names.into_iter().map(Cow::into_owned).collect(),
            debug_id: json.debug_id.map(Cow::into_owned),
            ignore_list: resolve_ignore_list(json.ignore_list, json.x_google_ignore_list)
                .map_err(serde::de::Error::custom)?,
        })
    }
}

fn deserialize_version<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let version = u32::deserialize(deserializer)?;
    if version != 3 {
        return Err(serde::de::Error::custom(format!("unsupported source map version: {version}")));
    }
    Ok(version)
}

pub fn decode(json: JSONSourceMap) -> Result<SourceMap<'static>> {
    if json.version != 3 {
        return Err(Error::BadJson(<serde_json::Error as serde::de::Error>::custom(format!(
            "unsupported source map version: {}",
            json.version
        ))));
    }
    validate_ignore_list(json.ignore_list.as_deref(), json.sources.len())?;

    let tokens = decode_mapping(&json.mappings, json.names.len(), json.sources.len())?;
    Ok(SourceMap {
        file: json.file.map(Cow::Owned),
        names: json.names.into_iter().map(Cow::Owned).collect(),
        source_root: json.source_root.map(Cow::Owned),
        sources: json.sources.into_iter().map(Cow::Owned).collect(),
        source_contents: json
            .sources_content
            .map(|content| content.into_iter().map(|c| c.map(Cow::Owned)).collect())
            .unwrap_or_default(),
        tokens: tokens.into_boxed_slice(),
        token_chunks: None,
        ignore_list: json.ignore_list,
        debug_id: json.debug_id.map(Cow::Owned),
    })
}

/// Private deserialization shape that borrows directly from the JSON input
/// when no escapes are present. The lifetime `'a` is the input buffer's
/// lifetime: each `Cow::Borrowed` is a zero-copy slice into that buffer, and
/// only escaped strings (which serde_json must unescape into a fresh `String`)
/// land in `Cow::Owned`.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BorrowedJSONSourceMap<'a> {
    #[serde(deserialize_with = "deserialize_version")]
    version: u32,
    #[serde(borrow)]
    file: Option<Cow<'a, str>>,
    #[serde(borrow)]
    mappings: Cow<'a, str>,
    #[serde(borrow)]
    source_root: Option<Cow<'a, str>>,
    #[serde(borrow)]
    sources: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    sources_content: Option<Vec<Option<Cow<'a, str>>>>,
    #[serde(borrow, default)]
    names: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    debug_id: Option<Cow<'a, str>>,
    ignore_list: Option<Vec<u32>>,
    // Defer legacy type validation until fallback is needed. The standard
    // `ignoreList` takes precedence, even if the legacy value is malformed.
    #[serde(rename = "x_google_ignoreList")]
    x_google_ignore_list: Option<serde_json::Value>,
}

fn resolve_ignore_list(
    ignore_list: Option<Vec<u32>>,
    x_google_ignore_list: Option<serde_json::Value>,
) -> serde_json::Result<Option<Vec<u32>>> {
    match ignore_list {
        Some(list) => Ok(Some(list)),
        None => x_google_ignore_list.map(serde_json::from_value).transpose(),
    }
}

pub fn decode_from_string(value: &str) -> Result<SourceMap<'_>> {
    let json: BorrowedJSONSourceMap<'_> = serde_json::from_str(value)?;

    let ignore_list = resolve_ignore_list(json.ignore_list, json.x_google_ignore_list)?;
    validate_ignore_list(ignore_list.as_deref(), json.sources.len())?;

    let tokens = decode_mapping(&json.mappings, json.names.len(), json.sources.len())?;

    Ok(SourceMap {
        file: json.file,
        names: json.names,
        source_root: json.source_root,
        sources: json.sources,
        source_contents: json.sources_content.unwrap_or_default(),
        tokens: tokens.into_boxed_slice(),
        token_chunks: None,
        ignore_list,
        debug_id: json.debug_id,
    })
}

fn validate_ignore_list(ignore_list: Option<&[u32]>, sources_len: usize) -> Result<()> {
    if let Some(ignore_list) = ignore_list {
        for &idx in ignore_list {
            if idx as usize >= sources_len {
                return Err(Error::BadSourceReference(idx));
            }
        }
    }
    Ok(())
}

fn decode_mapping(mapping: &str, names_len: usize, sources_len: usize) -> Result<Vec<Token>> {
    let mapping = mapping.as_bytes();

    let mut tokens: Vec<Token> = Vec::with_capacity(estimate_token_capacity(mapping));

    let mut dst_line = 0u32;
    let mut dst_col = 0u32;
    let mut src_id = 0;
    let mut src_line = 0;
    let mut src_col = 0;
    let mut name_id = 0;

    // Source map segments are delta-encoded and interpreted relative to previous values.
    //
    // Segment arity:
    // * 1 field: generated column
    // * 4 fields: generated column, source id, source line, source column
    // * 5 fields: 4-field segment + name id
    //
    // Delimiters:
    // * `,` separates segments on the same generated line
    // * `;` advances generated line and resets generated column
    let mut cursor = 0usize;
    let mut nums = [0i64; 5];
    while cursor < mapping.len() {
        match mapping[cursor] {
            b',' => {
                // Empty segment, skip.
                cursor += 1;
            }
            b';' => {
                // New destination line. Destination columns are line-relative.
                dst_line = dst_line.wrapping_add(1);
                dst_col = 0;
                cursor += 1;
            }
            _ => {
                let nums_len = parse_vlq_segment_into(mapping, &mut cursor, &mut nums)?;

                // `nums[0]` is always generated column delta.
                let new_dst_col = i64::from(dst_col) + nums[0];
                if new_dst_col < 0 {
                    return Err(Error::BadSegmentSize(0)); // Negative column
                }
                dst_col = new_dst_col as u32;

                let mut src = INVALID_ID;
                let mut name = INVALID_ID;

                if nums_len > 1 {
                    if nums_len != 4 && nums_len != 5 {
                        return Err(Error::BadSegmentSize(nums_len as u32));
                    }

                    // Source/name fields are also delta-encoded.
                    let new_src_id = i64::from(src_id) + nums[1];
                    if new_src_id < 0 || new_src_id >= sources_len as i64 {
                        return Err(Error::BadSourceReference(src_id));
                    }
                    src_id = new_src_id as u32;
                    src = src_id;

                    let new_src_line = i64::from(src_line) + nums[2];
                    if new_src_line < 0 {
                        return Err(Error::BadSegmentSize(0)); // Negative line
                    }
                    src_line = new_src_line as u32;

                    let new_src_col = i64::from(src_col) + nums[3];
                    if new_src_col < 0 {
                        return Err(Error::BadSegmentSize(0)); // Negative column
                    }
                    src_col = new_src_col as u32;

                    if nums_len > 4 {
                        name_id = (i64::from(name_id) + nums[4]) as u32;
                        if name_id >= names_len as u32 {
                            return Err(Error::BadNameReference(name_id));
                        }
                        name = name_id;
                    }
                }

                tokens.push(Token::new(
                    dst_line,
                    dst_col,
                    src_line,
                    src_col,
                    if src == INVALID_ID { None } else { Some(src) },
                    if name == INVALID_ID { None } else { Some(name) },
                ));
            }
        }
    }

    Ok(tokens)
}

/// Fast path for the dominant sourcemap segment shapes: a 4- or 5-field
/// segment whose VLQ values are all single-byte (no continuation bit),
/// terminated by `,` / `;` / end-of-input. Both arities are handled because
/// real bundler output splits roughly 60% 4-field / 25% 5-field (named)
/// segments; covering only one shape would send the other through the
/// general parser.
///
/// Fills `nums` and returns the field count (the caller advances `cursor`
/// by the same amount). Returns `None` without consuming anything when the
/// shape doesn't match, so the general parser can handle multi-byte VLQs,
/// other arities, and diagnostics.
///
/// Byte 4 is probed lazily, after the 4-lane decode, because its meaning is
/// what distinguishes the two arities: in a 4-field segment it is the
/// terminator (or absent at end of input), in a 5-field segment it is the
/// name field. It cannot join the packed lane test for the same reason, and
/// decoding all 5 lanes eagerly and branching afterwards benchmarked slightly
/// slower — the branch predictor already speculates into the 5th-field
/// handling when needed, so eager decoding only adds wasted work per
/// 4-field segment.
#[inline(always)]
fn try_fast_4_or_5_segment(mapping: &[u8], cursor: usize, nums: &mut [i64; 5]) -> Option<usize> {
    let rest = &mapping[cursor..];
    if rest.len() < 4 {
        return None;
    }

    let v = [
        B64_DECODE.0[rest[0] as usize] as u8,
        B64_DECODE.0[rest[1] as usize] as u8,
        B64_DECODE.0[rest[2] as usize] as u8,
        B64_DECODE.0[rest[3] as usize] as u8,
    ];
    // A table entry with any of the top three bits set (`& 0xE0`) is either a
    // delimiter (`-1` → `0xFF`) or has the VLQ continuation bit (32..=63)
    if (v[0] | v[1] | v[2] | v[3]) & 0xE0 != 0 {
        return None;
    }

    for i in 0..4 {
        nums[i] = decode_sign(i64::from(v[i]));
    }

    let Some(&b4) = rest.get(4) else { return Some(4) };
    let v4 = B64_DECODE.0[b4 as usize] as u8;
    if is_delimiter(v4) {
        return Some(4);
    }

    // the byte must be a single-byte 5th field with the segment ending right
    // after it, else fall back (6+ fields, or a multi-byte VLQ anywhere in
    // the segment).
    if v4 & 0xE0 != 0
        || rest.get(5).is_some_and(|&b5| !is_delimiter(B64_DECODE.0[b5 as usize] as u8))
    {
        return None;
    }
    nums[4] = decode_sign(i64::from(v4));
    Some(5)
}

/// Whether a [`B64_DECODE`] entry marks a segment (`,`) or line (`;`)
/// delimiter.
#[inline(always)]
fn is_delimiter(entry: u8) -> bool {
    // delimiters are the table's only negative entries
    entry & 0x80 != 0
}

// Align B64 lookup table on 64-byte boundary for better cache performance
#[repr(align(64))]
struct Aligned64([i8; 256]);

/// VLQ decode table. Entry semantics:
/// * `-1` — segment (`,`) or line (`;`) delimiter, terminates a value scan.
/// * `0..=63` — base64 payload; bit 5 (value >= 32) is the VLQ continuation flag.
///
/// Bytes outside the base64 alphabet decode as `63`, i.e. exactly like `'/'`
/// (payload 31 with the continuation bit set). This preserves the historical
/// lenient behavior: invalid characters are consumed as continuation bytes and
/// surface as `VlqLeftover`/`VlqOverflow` or garbage values, never a panic.
static B64_DECODE: Aligned64 = build_b64_decode();

const fn build_b64_decode() -> Aligned64 {
    let mut table = [63i8; 256];
    let mut i = 0;
    while i < 26 {
        table[(b'A' + i) as usize] = i as i8;
        table[(b'a' + i) as usize] = 26 + i as i8;
        i += 1;
    }
    let mut i = 0;
    while i < 10 {
        table[(b'0' + i) as usize] = 52 + i as i8;
        i += 1;
    }
    table[b'+' as usize] = 62;
    table[b'/' as usize] = 63;
    table[b',' as usize] = -1;
    table[b';' as usize] = -1;
    Aligned64(table)
}

/// Pick a `Vec<Token>` capacity for `decode_mapping`.
///
/// Two regimes:
/// * Small mappings (< 256 bytes) — exact `,`/`;` count. The scan itself is
///   cheap on a few hundred bytes, and getting capacity *exact* lets the
///   trailing `Vec::into_boxed_slice` in `decode_from_string` skip its
///   shrink-realloc. That realloc dominates the per-iteration cost on tiny
///   benchmarks (32-byte fixtures), so an over-estimate would regress them.
/// * Larger mappings — `len / 4 + 1`. The scan would itself become the
///   dominant cost (~15-20 µs on the xlarge perf fixture). The heuristic is
///   close to typical sourcemap density (~4-5 bytes per segment); `Vec::push`
///   handles under-estimates via geometric growth, and the trailing
///   `into_boxed_slice` shrinks any over-allocation back to exact size.
fn estimate_token_capacity(mapping: &[u8]) -> usize {
    const EXACT_SCAN_THRESHOLD: usize = 256;
    if mapping.len() < EXACT_SCAN_THRESHOLD {
        let mut n = 1usize;
        for &b in mapping {
            if b == b',' || b == b';' {
                n += 1;
            }
        }
        n
    } else {
        mapping.len() / 4 + 1
    }
}

/// Parse one VLQ segment at `cursor` and stop at `,` / `;` / end-of-input.
///
/// Returns the number of decoded values in the segment. Values are written into
/// `rv` for the first 5 fields (the maximum valid segment size). If the segment
/// contains more than 5 fields, we keep parsing and counting so the caller can
/// reject it with `BadSegmentSize` carrying the true field count.
fn parse_vlq_segment_into(mapping: &[u8], cursor: &mut usize, rv: &mut [i64; 5]) -> Result<usize> {
    // The dominant segment shape in real bundler output (~95%) is 4 or 5
    // single-byte VLQ values; decode it branch-light, leaving multi-byte
    // VLQs, other arities, and diagnostics to the general loop below.
    if let Some(seg_len) = try_fast_4_or_5_segment(mapping, *cursor, rv) {
        *cursor += seg_len;
        return Ok(seg_len);
    }

    let mut rv_len = 0usize;
    while let Some(value) = next_vlq(mapping, cursor)? {
        if rv_len < rv.len() {
            rv[rv_len] = value;
        }
        rv_len += 1;
    }
    if rv_len == 0 {
        return Err(Error::VlqNoValues);
    }
    Ok(rv_len)
}

/// Decode one VLQ value starting at `*cursor`.
///
/// Returns `Ok(None)` without consuming anything when positioned at a
/// delimiter (`,` / `;`) or end of input; otherwise consumes the value's bytes.
#[inline(always)]
fn next_vlq(mapping: &[u8], cursor: &mut usize) -> Result<Option<i64>> {
    let Some(&byte) = mapping.get(*cursor) else { return Ok(None) };
    let first = i64::from(B64_DECODE.0[byte as usize]);
    if first < 0 {
        return Ok(None);
    }
    *cursor += 1;
    if first < 32 {
        // No continuation bit: a single-byte value, the dominant case in real
        // mappings (small line/column deltas).
        return Ok(Some(decode_sign(first)));
    }

    let mut cur = first & 0b11111;
    let mut shift = 5u32;
    loop {
        let Some(&byte) = mapping.get(*cursor) else {
            // Input ended while a continuation was pending.
            return Err(Error::VlqLeftover);
        };
        let enc = i64::from(B64_DECODE.0[byte as usize]);
        if enc < 0 {
            // Delimiter while a continuation was pending.
            return Err(Error::VlqLeftover);
        }
        *cursor += 1;
        // VLQ shift grows by 5 bits per continuation byte. Bail out before
        // `payload << shift` could overflow i64: the largest safe shift for a
        // 5-bit value is 62, and `shift` only ever takes multiples of 5, so
        // the first offending shift is 65.
        if shift > 62 {
            return Err(Error::VlqOverflow);
        }
        cur |= (enc & 0b11111) << shift;
        shift += 5;
        if enc < 32 {
            return Ok(Some(decode_sign(cur)));
        }
    }
}

/// VLQ stores the sign in the low bit; remaining bits are the magnitude.
/// Branchless sign-magnitude decode: `-0` collapses to `0`.
#[inline(always)]
fn decode_sign(cur: i64) -> i64 {
    let mag = cur >> 1;
    let sign = cur & 1;
    (mag ^ -sign) + sign
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/source_map/src/decode/tests.rs"]
#[cfg(test)]
mod tests;
