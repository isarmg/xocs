//! Convert UTF-8 span offsets to UTF-16.

use crate::syntax_tree::ast::{Comment, Program};
use crate::source_location::Span;
use crate::syntax_types::module_record::{ModuleRecord, VisitMutModuleRecord};

mod converter;
mod translation;
mod visit;
pub use converter::Utf8ToUtf16Converter;
use translation::{Translation, build_translations};

/// Conversion table of UTF-8 span offsets to UTF-16.
pub struct Utf8ToUtf16 {
    translations: Vec<Translation>,
}

impl Utf8ToUtf16 {
    /// Create new [`Utf8ToUtf16`] conversion table from source text.
    pub fn new(source_text: &str) -> Self {
        let mut translations = Vec::with_capacity(16);

        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });

        build_translations(source_text, &mut translations, 0);

        // If no translations have been added after the first `0, 0` dummy, then source is entirely ASCII.
        // Remove the dummy entry.
        // Therefore, `translations` always has at least 2 entries, if it has any.
        if translations.len() == 1 {
            // In conformance tests, force offset conversion to happen on all inputs,
            // even if they are pure ASCII
            if cfg!(any()) {
                translations.push(Translation { utf8_offset: u32::MAX, utf16_difference: 0 });
            } else {
                translations.clear();
            }
        }

        Self { translations }
    }

    /// Create new [`Utf8ToUtf16`] conversion table from source text with an offset.
    ///
    /// `offset` is the number of bytes to subtract from UTF-8 offsets before converting to UTF-16.
    /// These bytes should not be part of `source_text` string.
    /// Offsets passed to the converter should be relative to the *untrimmed* source.
    ///
    /// If file starts with a BOM and UTF-16 offsets should be for the source text without the BOM,
    /// pass `source_text` with the BOM trimmed from the start, and `offset` as 3 (length of BOM in UTF-8 bytes).
    pub fn new_with_offset(source_text: &str, offset: u32) -> Self {
        let mut translations = Vec::with_capacity(16);

        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });
        translations.push(Translation { utf8_offset: offset, utf16_difference: offset });

        build_translations(source_text, &mut translations, offset);

        Self { translations }
    }

    /// Create a [`Utf8ToUtf16Converter`] converter, to convert offsets from UTF-8 to UTF-16.
    ///
    /// The converter is optimized for converting a sequence of offsets in ascending order.
    /// It will also correctly handle offsets in any order, but at a performance cost.
    ///
    /// Returns `None` if the source text is entirely ASCII, and so requires no conversion.
    pub fn converter(&self) -> Option<Utf8ToUtf16Converter<'_>> {
        if self.translations.is_empty() {
            None
        } else {
            // SAFETY: `translations` contains at least 2 entries if it's not empty.
            // We just checked it's not empty.
            Some(unsafe { Utf8ToUtf16Converter::new(&self.translations, false) })
        }
    }

    /// Convert all spans in AST to UTF-16.
    pub fn convert_program(&self, program: &mut Program<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.convert_program(program);
        }
    }

    /// Convert all spans in AST to UTF-16.
    ///
    /// Additionally, checks that conversion of offsets during traversal via [`Utf8ToUtf16Converter`]
    /// happens in ascending order of offset. Panics if it doesn't.
    ///
    /// This results in the fastest conversion, and [`Utf8ToUtf16Converter`] is designed to ensure that
    /// [`Utf8ToUtf16Converter::convert_offset`] is called with offsets strictly in ascending order.
    /// This should always be the case when the AST has come direct from parser.
    /// It might well not be the case in an AST which has been modified, e.g. by transformer or minifier.
    ///
    /// This method is for use only in conformance tests, and requires `conformance` Cargo feature.
    ///
    /// # Panics
    ///
    /// Panics if offsets are converted out of order.
    #[cfg(any())]
    pub fn convert_program_with_ascending_order_checks(&self, program: &mut Program<'_>) {
        assert!(self.translations.len() >= 2);

        // SAFETY: We just checked `translations` contains at least 2 entries
        let mut converter = unsafe { Utf8ToUtf16Converter::new(&self.translations, true) };
        converter.convert_program(program);
    }

    /// Convert all spans in AST and comments to UTF-16.
    pub fn convert_program_and_comments(&self, program: &mut Program<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.convert_program(program);

            converter.reset();

            for comment in &mut program.comments {
                converter.convert_span(&mut comment.span);
            }
        }
    }

    /// Convert all spans in comments to UTF-16.
    pub fn convert_comments(&self, comments: &mut [Comment]) {
        if let Some(mut converter) = self.converter() {
            for comment in comments {
                converter.convert_span(&mut comment.span);
            }
        }
    }

    /// Convert all spans in `ModuleRecord` to UTF-16.
    pub fn convert_module_record(&self, module_record: &mut ModuleRecord<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.visit_module_record(module_record);
        }
    }

    /// Convert a single UTF-16 offset back to UTF-8.
    pub fn convert_offset_back(&self, utf16_offset: &mut u32) {
        if let Some(converter) = self.converter() {
            converter.convert_offset_back(utf16_offset);
        }
    }

    /// Convert [`Span`] from UTF-16 offsets to UTF-8 offsets.
    pub fn convert_span_back(&self, span: &mut Span) {
        if let Some(converter) = self.converter() {
            converter.convert_span_back(span);
        }
    }
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/syntax_visitor/src/utf8_to_utf16/mod/test.rs"]
#[cfg(test)]
mod test;
