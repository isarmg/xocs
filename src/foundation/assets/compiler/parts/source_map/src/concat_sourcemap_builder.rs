use std::borrow::Cow;

use crate::source_map::{SourceMap, Token, token::TokenChunk};

/// The `ConcatSourceMapBuilder` is a helper to concat sourcemaps.
///
/// Names/sources/sourcesContent are accumulated as [`Cow<'a, str>`], so the builder serves two
/// input shapes without ever copying string bytes during the join:
///
/// * [`add_sourcemap`](Self::add_sourcemap) / [`from_sourcemaps`](Self::from_sourcemaps) **borrow**
///   from input maps that outlive the builder (`Cow::Borrowed`).
/// * [`add_sourcemap_owned`](Self::add_sourcemap_owned) /
///   [`from_owned_sourcemaps`](Self::from_owned_sourcemaps) **consume** owned input maps and
///   **move** their strings in (`Cow::Owned`) — no copy, ideal when the inputs are discarded
///   afterwards (e.g. a bundler joining freshly-rendered modules).
///
/// The ownership decision is deferred to the end:
/// * [`into_sourcemap`](Self::into_sourcemap) moves the accumulated entries straight into a
///   [`SourceMap<'a>`] — zero copy either way.
/// * [`into_owned_sourcemap`](Self::into_owned_sourcemap) detaches to `'static`: entries moved in
///   via `add_sourcemap_owned` are kept as-is, only borrowed ones are copied.
#[derive(Debug, Default)]
pub struct ConcatSourceMapBuilder<'a> {
    pub(crate) names: Vec<Cow<'a, str>>,
    pub(crate) sources: Vec<Cow<'a, str>>,
    pub(crate) source_contents: Vec<Option<Cow<'a, str>>>,
    pub(crate) tokens: Vec<Token>,
    /// The `token_chunks` is used for encode tokens to vlq mappings at parallel.
    pub(crate) token_chunks: Vec<TokenChunk>,
    pub(crate) token_chunk_prev_source_id: u32,
    pub(crate) token_chunk_prev_name_id: u32,
}

impl<'a> ConcatSourceMapBuilder<'a> {
    /// Create new `ConcatSourceMapBuilder` with pre-allocated capacity.
    ///
    /// Allocating capacity before adding sourcemaps with `add_sourcemap` avoids memory copies
    /// and increases performance.
    ///
    /// Alternatively, use `from_sourcemaps`.
    pub fn with_capacity(
        names_len: usize,
        sources_len: usize,
        tokens_len: usize,
        token_chunks_len: usize,
    ) -> Self {
        Self {
            names: Vec::with_capacity(names_len),
            sources: Vec::with_capacity(sources_len),
            source_contents: Vec::with_capacity(sources_len),
            tokens: Vec::with_capacity(tokens_len),
            token_chunks: Vec::with_capacity(token_chunks_len),
            token_chunk_prev_source_id: 0,
            token_chunk_prev_name_id: 0,
        }
    }

    /// Sum the `names` / `sources` / `tokens` lengths across `maps`, to pre-size the builder.
    fn sum_lengths<'s, 'd: 's>(
        maps: impl Iterator<Item = &'s SourceMap<'d>>,
    ) -> (usize, usize, usize) {
        maps.fold((0, 0, 0), |(names, sources, tokens), map| {
            (names + map.names.len(), sources + map.sources.len(), tokens + map.tokens.len())
        })
    }

    /// Pad `source_contents` with `None` so it stays index-aligned with `sources`. An input map's
    /// `sourcesContent` may be absent or shorter than its `sources` (it is not normalized on
    /// decode), and contents are indexed by source id — without this, a later map's content would
    /// shift onto an earlier map's source. A no-op when the map's contents already cover its
    /// sources (the common case), so it costs nothing there.
    fn pad_source_contents(&mut self) {
        self.source_contents.resize(self.sources.len(), None);
    }

    /// Create new `ConcatSourceMapBuilder` from an array of `SourceMap`s and line offsets.
    ///
    /// This avoids memory copies versus creating builder with `ConcatSourceMapBuilder::default()`
    /// and then adding sourcemaps individually with `add_sourcemap`.
    ///
    /// # Example
    /// ```no_run
    /// # use oxc_sourcemap::ConcatSourceMapBuilder;
    /// # use oxc_sourcemap::SourceMap;
    /// # let sourcemap1 = SourceMap::default();
    /// # let sourcemap2 = SourceMap::default();
    /// let builder = ConcatSourceMapBuilder::from_sourcemaps(&[
    ///   (&sourcemap1, 0),
    ///   (&sourcemap2, 100),
    /// ]);
    /// let combined_sourcemap = builder.into_sourcemap();
    /// ```
    pub fn from_sourcemaps(sourcemap_and_line_offsets: &[(&'a SourceMap<'_>, u32)]) -> Self {
        let (names_len, sources_len, tokens_len) =
            Self::sum_lengths(sourcemap_and_line_offsets.iter().map(|(sourcemap, _)| *sourcemap));
        let mut builder = Self::with_capacity(
            names_len,
            sources_len,
            tokens_len,
            sourcemap_and_line_offsets.len(),
        );

        for &(sourcemap, line_offset) in sourcemap_and_line_offsets {
            builder.add_sourcemap(sourcemap, line_offset);
        }

        builder
    }

    /// Create a `ConcatSourceMapBuilder` from **owned** `SourceMap`s and line offsets, **moving**
    /// their strings in (no copy).
    ///
    /// The owned counterpart of [`from_sourcemaps`](Self::from_sourcemaps) — see
    /// [`add_sourcemap_owned`](Self::add_sourcemap_owned). This is the fastest way to join owned
    /// maps that are discarded afterwards (e.g. a bundler joining rendered modules): no
    /// name/source/sourcesContent string is copied.
    pub fn from_owned_sourcemaps(sourcemap_and_line_offsets: Vec<(SourceMap<'a>, u32)>) -> Self {
        let (names_len, sources_len, tokens_len) =
            Self::sum_lengths(sourcemap_and_line_offsets.iter().map(|(sourcemap, _)| sourcemap));
        let mut builder = Self::with_capacity(
            names_len,
            sources_len,
            tokens_len,
            sourcemap_and_line_offsets.len(),
        );

        for (sourcemap, line_offset) in sourcemap_and_line_offsets {
            builder.add_sourcemap_owned(sourcemap, line_offset);
        }

        builder
    }

    /// Add a **borrowed** `SourceMap`'s entries to the concatenation, offset by `line_offset`.
    ///
    /// Strings are borrowed from `sourcemap` for `'a` (no allocation), so the result cannot
    /// outlive it. Use [`add_sourcemap_owned`](Self::add_sourcemap_owned) when you own the map and
    /// want to move its strings in instead.
    pub fn add_sourcemap(&mut self, sourcemap: &'a SourceMap<'_>, line_offset: u32) {
        let source_offset = self.sources.len() as u32;
        let name_offset = self.names.len() as u32;

        // Borrow strings directly from the input map — no allocations. The output `SourceMap`'s
        // lifetime is tied to `'a`, so the borrow checker enforces that input maps outlive it.
        self.sources.extend(sourcemap.get_sources().map(Cow::Borrowed));
        self.source_contents
            .extend(sourcemap.get_source_contents().map(|content| content.map(Cow::Borrowed)));
        self.pad_source_contents();
        self.names.extend(sourcemap.get_names().map(Cow::Borrowed));

        self.add_tokens(&sourcemap.tokens, line_offset, source_offset, name_offset);
    }

    /// Add an **owned** `SourceMap` to the concatenation, **moving** its strings in (no copy),
    /// offset by `line_offset`.
    ///
    /// This is the fastest path when the inputs are owned and discarded after the join: the
    /// name/source/sourcesContent `Cow::Owned` heap buffers are moved across rather than copied,
    /// so the cost is independent of how much `sourcesContent` rides along. Pair with
    /// [`into_sourcemap`](Self::into_sourcemap), which then moves the entries straight into the
    /// result (and, for `'static` inputs, returns a `'static` map without any copy).
    pub fn add_sourcemap_owned(&mut self, sourcemap: SourceMap<'a>, line_offset: u32) {
        let source_offset = self.sources.len() as u32;
        let name_offset = self.names.len() as u32;

        let parts = sourcemap.into_parts();

        // Move the owned entries in — no string bytes are copied.
        self.sources.extend(parts.sources);
        self.source_contents.extend(parts.source_contents);
        self.pad_source_contents();
        self.names.extend(parts.names);

        self.add_tokens(&parts.tokens, line_offset, source_offset, name_offset);
    }

    /// Append `tokens` to `self.tokens`, translated by `line_offset` / `source_offset` /
    /// `name_offset` so they resolve against the combined `sources` / `names` arrays, and record
    /// the matching [`TokenChunk`]. Shared by `add_sourcemap` (borrowed) and `add_sourcemap_owned`
    /// (owned) — only how the strings get in differs.
    fn add_tokens(
        &mut self,
        tokens: &[Token],
        line_offset: u32,
        source_offset: u32,
        name_offset: u32,
    ) {
        let start = self.tokens.len();
        // The chunk header records the prev-id baseline as it stood *before* this chunk.
        let chunk_prev_source_id = self.token_chunk_prev_source_id;
        let chunk_prev_name_id = self.token_chunk_prev_name_id;

        if start == 0 && line_offset == 0 && source_offset == 0 && name_offset == 0 {
            // Genuinely the first contributing map: no line/source/name offset, and no previous
            // token to dedup against, so every token is unchanged — copy them in one `memcpy`.
            // (A prior map can add sources/names without tokens, leaving `start == 0` while the
            // offsets are non-zero, so all four conditions must hold to skip translation.)
            self.tokens.extend_from_slice(tokens);
        } else {
            self.tokens.reserve(tokens.len());
            let mut tokens = tokens.iter();
            // Boundary dedup: only the first token can equal the previous map's last token (every
            // later token has a distinct generated position), so check it once and drop if equal.
            if let Some(first) = tokens.next() {
                let first = first.translated(line_offset, source_offset, name_offset);
                if self.tokens.last() != Some(&first) {
                    self.tokens.push(first);
                }
            }
            self.tokens.extend(
                tokens.map(|token| token.translated(line_offset, source_offset, name_offset)),
            );
        }

        // The next chunk's VLQ baseline is the last source/name id committed. Scan back from the
        // end of what we just appended — the final token almost always carries both, so this is
        // typically O(1); if this map contributed neither, the previous baseline carries over.
        let (mut prev_source_id, mut prev_name_id) = (None, None);
        for token in self.tokens[start..].iter().rev() {
            prev_source_id = prev_source_id.or_else(|| token.get_source_id());
            prev_name_id = prev_name_id.or_else(|| token.get_name_id());
            if prev_source_id.is_some() && prev_name_id.is_some() {
                break;
            }
        }
        self.token_chunk_prev_source_id = prev_source_id.unwrap_or(chunk_prev_source_id);
        self.token_chunk_prev_name_id = prev_name_id.unwrap_or(chunk_prev_name_id);

        // Record the chunk once boundary dedup has settled the actual end index.
        let end = self.tokens.len() as u32;
        let chunk = if start > 0 {
            let prev = &self.tokens[start - 1];
            TokenChunk::new(
                start as u32,
                end,
                prev.get_dst_line(),
                prev.get_dst_col(),
                prev.get_src_line(),
                prev.get_src_col(),
                chunk_prev_name_id,
                chunk_prev_source_id,
            )
        } else {
            TokenChunk::new(0, end, 0, 0, 0, 0, 0, 0)
        };
        self.token_chunks.push(chunk);
    }

    /// Finish, moving the accumulated names/sources/contents straight into a [`SourceMap<'a>`]
    /// (zero copy — the `Cow` vectors are moved, not rebuilt).
    pub fn into_sourcemap(self) -> SourceMap<'a> {
        SourceMap::new(
            None,
            self.names,
            None,
            self.sources,
            self.source_contents,
            self.tokens.into_boxed_slice(),
            Some(self.token_chunks),
        )
    }

    /// Same as [`Self::into_sourcemap`], but detaches to a `'static` [`crate::source_map::OwnedSourceMap`].
    /// Entries moved in via [`add_sourcemap_owned`](Self::add_sourcemap_owned) are kept as-is
    /// (no copy); only borrowed entries (from [`add_sourcemap`](Self::add_sourcemap)) are copied.
    #[inline]
    pub fn into_owned_sourcemap(self) -> crate::source_map::OwnedSourceMap {
        self.into_sourcemap().into_owned_sourcemap()
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/source_map/src/concat_sourcemap_builder/tests.rs"]
#[cfg(test)]
mod tests;
