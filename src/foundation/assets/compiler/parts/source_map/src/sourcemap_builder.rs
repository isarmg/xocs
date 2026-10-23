use std::borrow::Cow;

use rustc_hash::FxHashMap;

use crate::source_map::{
    SourceMap,
    token::{Token, TokenChunk},
};

/// Helper to build a [`SourceMap`].
///
/// The builder **borrows** the names, sources and source contents you add for its lifetime `'a`,
/// so building allocates essentially nothing beyond the tokens vector (the dedup maps key by
/// `&'a str`, not owned copies).
///
/// The ownership decision is deferred to the end:
/// * [`into_sourcemap`](Self::into_sourcemap) returns a borrowed [`SourceMap<'a>`] — zero copy.
/// * [`into_owned_sourcemap`](Self::into_owned_sourcemap) copies the strings once into a
///   `'static` [`crate::source_map::OwnedSourceMap`].
#[derive(Debug, Default)]
pub struct SourceMapBuilder<'a> {
    pub(crate) file: Option<&'a str>,
    pub(crate) names_map: FxHashMap<&'a str, u32>,
    pub(crate) names: Vec<&'a str>,
    pub(crate) sources: Vec<&'a str>,
    pub(crate) sources_map: FxHashMap<&'a str, u32>,
    pub(crate) source_contents: Vec<Option<&'a str>>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) token_chunks: Option<Vec<TokenChunk>>,
}

impl<'a> SourceMapBuilder<'a> {
    /// Add a name, deduplicating. The name is borrowed for `'a` (no allocation).
    pub fn add_name(&mut self, name: &'a str) -> u32 {
        *self.names_map.entry(name).or_insert_with(|| {
            let id = self.names.len() as u32;
            self.names.push(name);
            id
        })
    }

    /// Add a source and its content, deduplicating on the source path.
    /// Both are borrowed for `'a` (no allocation). Use this if `source` may be a duplicate.
    pub fn add_source_and_content(&mut self, source: &'a str, source_content: &'a str) -> u32 {
        *self.sources_map.entry(source).or_insert_with(|| {
            let id = self.sources.len() as u32;
            self.sources.push(source);
            self.source_contents.push(Some(source_content));
            id
        })
    }

    /// Add a source and its content without deduplicating (skips the hash lookup when sources
    /// are unique).
    ///
    /// The source name and source content are borrowed for `'a` — neither is copied.
    pub fn set_source_and_content(&mut self, source: &'a str, source_content: &'a str) -> u32 {
        let count = self.sources.len() as u32;
        self.sources.push(source);
        self.source_contents.push(Some(source_content));
        count
    }

    /// Add item to `SourceMap::tokens`.
    pub fn add_token(
        &mut self,
        dst_line: u32,
        dst_col: u32,
        src_line: u32,
        src_col: u32,
        src_id: Option<u32>,
        name_id: Option<u32>,
    ) {
        self.tokens.push(Token::new(dst_line, dst_col, src_line, src_col, src_id, name_id));
    }

    /// Set the generated file name. Borrowed for `'a` (no allocation).
    pub fn set_file(&mut self, file: &'a str) {
        self.file = Some(file);
    }

    /// Set the `SourceMap::token_chunks` to make the sourcemap to vlq mapping at parallel.
    pub fn set_token_chunks(&mut self, token_chunks: Vec<TokenChunk>) {
        self.token_chunks = Some(token_chunks);
    }

    /// Finish, borrowing the names/sources/contents for `'a` (zero copy).
    pub fn into_sourcemap(mut self) -> SourceMap<'a> {
        // Names and sources are collected into new vectors below, and converting tokens
        // into a boxed slice already discards their spare capacity.
        if let Some(c) = self.token_chunks.as_mut() {
            c.shrink_to_fit()
        }
        SourceMap::new(
            self.file.map(Cow::Borrowed),
            self.names.into_iter().map(Cow::Borrowed).collect(),
            None,
            self.sources.into_iter().map(Cow::Borrowed).collect(),
            self.source_contents.into_iter().map(|content| content.map(Cow::Borrowed)).collect(),
            self.tokens.into_boxed_slice(),
            self.token_chunks,
        )
    }

    /// Same as [`Self::into_sourcemap`], but copies the strings once into an owned
    /// [`crate::source_map::OwnedSourceMap`] so callers can store the result without spelling out `'static`.
    #[inline]
    pub fn into_owned_sourcemap(self) -> crate::source_map::OwnedSourceMap {
        self.into_sourcemap().into_owned_sourcemap()
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/source_map/src/sourcemap_builder/tests.rs"]
#[cfg(test)]
mod tests;
