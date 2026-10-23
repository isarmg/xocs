use bitflags::bitflags;
use crate::compiler_errors::Result;

use crate::regular_expression::{
    diagnostics,
    parser::{reader::Reader, span_factory::SpanFactory},
};

bitflags! {
    /// Regular expression flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u8 {
        /// Indices flag
        const D = 1 << 0;
        /// Global flag
        const G = 1 << 1;
        /// Ignore case flag
        const I = 1 << 2;
        /// Multiline flag
        const M = 1 << 3;
        /// DotAll flag
        const S = 1 << 4;
        /// Unicode flag
        const U = 1 << 5;
        /// Unicode sets flag
        const V = 1 << 6;
        /// Sticky flag
        const Y = 1 << 7;
    }
}

impl TryFrom<char> for Flags {
    type Error = ();

    fn try_from(c: char) -> std::result::Result<Self, Self::Error> {
        match c {
            'd' => Ok(Self::D),
            'g' => Ok(Self::G),
            'i' => Ok(Self::I),
            'm' => Ok(Self::M),
            's' => Ok(Self::S),
            'u' => Ok(Self::U),
            'v' => Ok(Self::V),
            'y' => Ok(Self::Y),
            _ => Err(()),
        }
    }
}

pub struct FlagsParser<'a> {
    reader: Reader<'a>,
    span_factory: SpanFactory,
}

impl<'a> FlagsParser<'a> {
    pub fn new(reader: Reader<'a>, span_offset: u32) -> Self {
        Self { reader, span_factory: SpanFactory::new(span_offset) }
    }

    /// Returns: (is_unicode_mode, is_unicode_sets_mode)
    pub fn parse(mut self) -> Result<(bool, bool)> {
        let mut is_unicode_mode = false;
        let mut is_unicode_sets_mode = false;
        let mut seen = Flags::empty();

        while let Some(cp) = self.reader.peek() {
            let span_start = self.reader.offset();
            self.reader.advance();
            let span_end = self.reader.offset();

            let flag =
                char::try_from(cp).ok().and_then(|c| Flags::try_from(c).ok()).ok_or_else(|| {
                    diagnostics::unknown_flag(
                        self.span_factory.create(span_start, span_end),
                        &self.reader.str(span_start, span_end),
                        &["d", "g", "i", "m", "s", "u", "v", "y"],
                    )
                })?;

            if seen.contains(flag) {
                return Err(diagnostics::duplicated_flags(
                    self.span_factory.create(span_start, span_end),
                    &self.reader.str(span_start, span_end),
                ));
            }

            if flag == Flags::U {
                if seen.contains(Flags::V) {
                    return Err(diagnostics::invalid_unicode_flags(
                        self.span_factory.create(span_start, span_end),
                    ));
                }
                is_unicode_mode = true;
            } else if flag == Flags::V {
                if seen.contains(Flags::U) {
                    return Err(diagnostics::invalid_unicode_flags(
                        self.span_factory.create(span_start, span_end),
                    ));
                }
                is_unicode_mode = true;
                is_unicode_sets_mode = true;
            }

            seen.insert(flag);
        }

        Ok((is_unicode_mode, is_unicode_sets_mode))
    }
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/regular_expression/src/parser/flags_parser/test.rs"]
#[cfg(test)]
mod test;
