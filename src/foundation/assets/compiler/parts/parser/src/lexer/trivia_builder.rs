use std::num::NonZeroU32;

use memchr::memchr_iter;

use crate::syntax_arena::{Allocator, ArenaVec};
use crate::syntax_tree::ast::{Comment, CommentContent, CommentKind, CommentPosition};
use crate::source_location::Span;

use super::{Kind, Token};

#[derive(Debug)]
pub struct TriviaBuilder<'a> {
    // This is a set of unique comments. Duplicated
    // comments could be generated in case of rewind; they are
    // filtered out at insertion time.
    pub(crate) comments: ArenaVec<'a, Comment>,

    pub(crate) irregular_whitespaces: Vec<Span>,

    // states
    /// index of processed comments
    processed: usize,

    /// Saw a newline before this position (since last token).
    /// Used to determine if comments are trailing comments of the previous token.
    saw_newline: bool,

    /// Saw a newline before this position (since last comment or token).
    /// Used to set `preceded_by_newline` on comments.
    saw_newline_for_comment: bool,

    /// Previous token, used to attach trailing comments to its end.
    previous_token: Token,

    /// Range of comments preceding the current token that contains pure comments.
    ///
    /// `(start, end)` indexes into `comments`, with `end` exclusive.
    /// `end` is always at least `start + 1`, so it is a `NonZeroU32`.
    /// That gives the `Option` a niche, so the field is 8 bytes.
    pure_comments: Option<(u32, NonZeroU32)>,

    /// Range of comments preceding the current token that contains no-side-effects comments.
    /// Same representation as `pure_comments`.
    no_side_effects_comments: Option<(u32, NonZeroU32)>,
}

const _: () = assert!(size_of::<Option<(u32, NonZeroU32)>>() == 8);

impl<'a> TriviaBuilder<'a> {
    pub fn new_in(allocator: &'a Allocator) -> Self {
        let mut previous_token = Token::default();
        previous_token.set_kind(Kind::Undetermined);
        Self {
            comments: ArenaVec::new_in(&allocator),
            irregular_whitespaces: vec![],
            processed: 0,
            saw_newline: true,
            saw_newline_for_comment: true,
            previous_token,
            pure_comments: None,
            no_side_effects_comments: None,
        }
    }

    pub fn previous_token_pure_comments(&self) -> Option<(u32, NonZeroU32)> {
        self.pure_comments
    }

    pub fn previous_token_no_side_effects_comments(&self) -> Option<(u32, NonZeroU32)> {
        self.no_side_effects_comments
    }

    pub(super) fn set_pure_comments(&mut self, pure_comments: Option<(u32, NonZeroU32)>) {
        self.pure_comments = pure_comments;
    }

    pub(super) fn clear_pure_comments(&mut self) {
        self.pure_comments = None;
    }

    pub(super) fn set_no_side_effects_comments(&mut self, comments: Option<(u32, NonZeroU32)>) {
        self.no_side_effects_comments = comments;
    }

    pub(super) fn clear_no_side_effects_comments(&mut self) {
        self.no_side_effects_comments = None;
    }

    pub fn mark_pure_comments_applied(&mut self, (start, end): (u32, NonZeroU32)) {
        for comment in &mut self.comments[start as usize..end.get() as usize] {
            if comment.content == CommentContent::PureNotApplied {
                comment.content = CommentContent::Pure;
            }
        }
    }

    pub fn mark_no_side_effects_comments_applied(&mut self, (start, end): (u32, NonZeroU32)) {
        for comment in &mut self.comments[start as usize..end.get() as usize] {
            if comment.content == CommentContent::NoSideEffectsNotApplied {
                comment.content = CommentContent::NoSideEffects;
            }
        }
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        // The irregular whitespaces array is ordered; only add if not added before, to avoid
        // duplicates when the parser looks ahead (e.g. `peek_token`) and rewinds, then re-lexes the
        // same whitespace. Same approach as `add_comment`.
        if let Some(last) = self.irregular_whitespaces.last()
            && start <= last.start
        {
            return;
        }
        self.irregular_whitespaces.push(Span::new(start, end));
    }

    pub fn add_line_comment(&mut self, start: u32, end: u32, kind: CommentKind, source_text: &str) {
        self.add_comment(Comment::new(start, end, kind), source_text);
    }

    pub fn add_block_comment(
        &mut self,
        start: u32,
        end: u32,
        kind: CommentKind,
        source_text: &str,
    ) {
        self.add_comment(Comment::new(start, end, kind), source_text);
    }

    // For block comments only. This function is not called after line comments because the lexer skips
    // newline after line comments.
    pub fn handle_newline(&mut self) {
        // The last unprocessed comment is on a newline.
        let len = self.comments.len();
        if self.processed < len {
            let becomes_trailing = {
                let comment = &mut self.comments[len - 1];
                comment.set_followed_by_newline(true);
                !self.saw_newline && !Self::should_stay_leading(comment)
            };
            if becomes_trailing {
                self.attach_pending_comments(
                    CommentPosition::Trailing,
                    self.previous_token.end(),
                    len,
                );
            }
        }
        self.saw_newline = true;
        self.saw_newline_for_comment = true;
    }

    #[inline]
    pub fn handle_token(&mut self, token: Token) {
        self.saw_newline = false;
        self.saw_newline_for_comment = false;
        // Cold path: attach any unprocessed comments between the previous token and this one.
        // For files with no comments (or once all comments are consumed), this branch is skipped.
        let len = self.comments.len();
        if self.processed < len {
            self.attach_pending_comments_to_token(token, len);
        }
        self.previous_token = token;
    }

    #[cold]
    fn attach_pending_comments_to_token(&mut self, token: Token, len: usize) {
        let previous_kind = self.previous_token.kind();
        let can_attach_to_previous = previous_kind != Kind::Undetermined
            && (token.kind() == Kind::Eof
                || (Self::can_end_expression(previous_kind)
                    && Self::continues_previous_expression(token.kind())));
        let previous_token_end = self.previous_token.end();
        for comment in &mut self.comments[self.processed..len] {
            if can_attach_to_previous
                && !comment.preceded_by_newline()
                && !Self::should_stay_leading(comment)
            {
                comment.position = CommentPosition::Trailing;
                comment.attached_to = previous_token_end;
            } else {
                comment.position = CommentPosition::Leading;
                comment.attached_to = token.start();
            }
        }
        self.processed = len;
    }

    #[cold]
    fn attach_pending_comments(&mut self, position: CommentPosition, attached_to: u32, len: usize) {
        for comment in &mut self.comments[self.processed..len] {
            comment.position = position;
            comment.attached_to = attached_to;
        }
        self.processed = len;
    }

    /// Whether a token can form the completed left-hand side of an infix or suffix boundary.
    #[inline]
    fn can_end_expression(kind: Kind) -> bool {
        kind.is_identifier()
            || kind.is_literal()
            || matches!(
                kind,
                Kind::PrivateIdentifier
                    | Kind::This
                    | Kind::Super
                    | Kind::RParen
                    | Kind::RBrack
                    | Kind::RCurly
                    | Kind::RAngle
                    | Kind::Plus2
                    | Kind::Minus2
                    | Kind::NoSubstitutionTemplate
                    | Kind::TemplateTail
                    | Kind::JSXText
            )
    }

    /// Whether trivia before `kind` belongs after a completed expression rather than before the
    /// next operand. This only runs when comments are present; parser hot paths are unaffected.
    #[inline]
    fn continues_previous_expression(kind: Kind) -> bool {
        kind.is_binary_operator() || kind.is_logical_operator() || kind.is_assignment_operator()
    }

    /// Determines if the current line comment should be treated as a trailing comment.
    ///
    /// A line comment should be treated as trailing when both of the following conditions are met:
    ///
    /// 1. It is not preceded by a newline.
    ///
    /// ```javascript
    /// let x = 5; // This should be treated as a trailing comment
    /// foo(); // This should also be treated as a trailing comment
    ///
    /// // This should not be treated as trailing (preceded by newline)
    /// let x = 5;
    /// ```
    ///
    /// 2. It does not immediately follow an `=` [`Kind::Eq`], `(` [`Kind::LParen`]
    ///    or `:` [`Kind::Colon`] token.
    ///
    /// ```javascript
    /// let y = // This should not be treated as trailing (follows `=`)
    ///     10;
    ///
    /// function foo( // This should not be treated as trailing (follows `(`)
    ///     param
    /// ) {}
    ///
    /// let z = cond ? a : // This should not be treated as trailing (follows `:`)
    ///     b;
    /// ```
    ///
    /// Treating a comment after `:` as trailing drops it (it anchors to the
    /// previous token rather than the following operand), which breaks codegen
    /// idempotency once a transform emits `? consequent : // comment\nalternate`.
    fn should_be_treated_as_trailing_comment(&self) -> bool {
        !self.saw_newline
            && !matches!(self.previous_token.kind(), Kind::Eq | Kind::LParen | Kind::Colon)
    }

    fn should_stay_leading(comment: &Comment) -> bool {
        // Match esbuild's model where legal comments are preserved before the following token/statement.
        // Annotation comments (`@__PURE__`, `@__NO_SIDE_EFFECTS__`) semantically mark the *next*
        // token, so they must also stay leading even when no newline precedes them — otherwise
        // codegen's minified output (which smashes statements together) breaks idempotency:
        // pass 1 emits the verbatim annotation as leading, pass 2 re-parses it as trailing of the
        // previous `}`/`;` and loses the `attached_to`, falling back to the canonical literal.
        matches!(
            comment.content,
            CommentContent::Legal
                | CommentContent::JsdocLegal
                | CommentContent::Pure
                | CommentContent::PureNotApplied
                | CommentContent::NoSideEffects
                | CommentContent::NoSideEffectsNotApplied
                | CommentContent::PropertyKey
        )
    }

    /// Update annotation state for the comment at `index`.
    fn set_annotation_flags(&mut self, comment: &Comment, index: usize) {
        let range = match comment.content {
            CommentContent::PureNotApplied => &mut self.pure_comments,
            CommentContent::NoSideEffectsNotApplied => &mut self.no_side_effects_comments,
            _ => return,
        };

        // Comment count cannot exceed `u32::MAX` because of `MAX_LEN` check in `Source::new`,
        // so `index + 1` cannot overflow, and is never zero
        #[expect(clippy::cast_possible_truncation)]
        let index = index as u32;
        let end = NonZeroU32::new(index + 1).unwrap();
        if let Some((_, range_end)) = range {
            *range_end = end;
        } else {
            *range = Some((index, end));
        }
    }

    fn add_comment(&mut self, mut comment: Comment, source_text: &str) {
        Self::parse_annotation(&mut comment, source_text);
        // The comments array is an ordered vec, only add the comment if its not added before,
        // to avoid situations where the parser needs to rewind and tries to reinsert the comment.
        if let Some(last_comment) = self.comments.last()
            && comment.span.start <= last_comment.span.start
        {
            // Duplicate from parser lookahead/rewind — update annotation flags
            // to point to the existing comment.
            if let Ok(index) = self
                .comments
                .binary_search_by_key(&comment.span.start, |existing| existing.span.start)
                && self.comments[index].span == comment.span
            {
                self.set_annotation_flags(&comment, index);
            }
            return;
        }

        // This newly added comment may be preceded by a newline.
        // Use `saw_newline_for_comment` which tracks newlines since the last comment or token,
        // not just since the last token.
        comment.set_preceded_by_newline(self.saw_newline_for_comment);
        let becomes_trailing = if comment.is_line() {
            // A line comment is always followed by a newline. This is never set in `handle_newline`.
            comment.set_followed_by_newline(true);
            let becomes_trailing = self.should_be_treated_as_trailing_comment()
                && !Self::should_stay_leading(&comment);
            self.saw_newline = true;
            self.saw_newline_for_comment = true;
            becomes_trailing
        } else {
            // Block comments don't end with a newline, so reset saw_newline_for_comment.
            // If there's a newline after the block comment, `handle_newline` will set it back to true.
            self.saw_newline_for_comment = false;
            false
        };

        // Set annotation flags here (not in `parse_annotation`) so the index is correct
        // even when the dedup check above skips a duplicate from parser lookahead/rewind.
        self.set_annotation_flags(&comment, self.comments.len());
        self.comments.push(comment);
        if becomes_trailing {
            self.attach_pending_comments(
                CommentPosition::Trailing,
                self.previous_token.end(),
                self.comments.len(),
            );
        }
    }

    /// Parse Notation
    fn parse_annotation(comment: &mut Comment, source_text: &str) {
        let s = comment.content_span().source_text(source_text);
        let bytes = s.as_bytes();

        // Early exit for empty comments
        if bytes.is_empty() {
            return;
        }

        // Check first byte for quick routing
        match bytes[0] {
            b'!' => {
                comment.content = CommentContent::Legal;
                return;
            }
            b'*' if comment.is_block() => {
                // Ignore webpack comment `/*****/`
                if !bytes.iter().all(|&c| c == b'*') {
                    if contains_license_or_preserve_comment(s) {
                        comment.content = CommentContent::JsdocLegal;
                    } else {
                        comment.content = CommentContent::Jsdoc;
                    }
                }
                return;
            }
            _ => {}
        }

        // Skip leading whitespace without allocation
        let mut start = 0;
        while start < bytes.len() && bytes[start].is_ascii_whitespace() {
            start += 1;
        }

        if start >= bytes.len() {
            return;
        }

        let rest = &bytes[start..];
        if (rest.starts_with(b"@__KEY__") || rest.starts_with(b"#__KEY__") || !rest[0].is_ascii())
            && is_property_key_annotation(s)
        {
            comment.content = CommentContent::PropertyKey;
            return;
        }

        // Fast path: check first non-whitespace byte
        match bytes[start] {
            b'@' => {
                start += 1;
                if start >= bytes.len() {
                    return;
                }

                // Check for @vite, @license, @preserve
                if bytes[start..].starts_with(b"vite") {
                    comment.content = CommentContent::Vite;
                    return;
                }
                if bytes[start..].starts_with(b"license") || bytes[start..].starts_with(b"preserve")
                {
                    comment.content = CommentContent::Legal;
                    return;
                }

                // Continue to check for __PURE__ or __NO_SIDE_EFFECTS__ after @
            }
            b'#' => {
                start += 1;
                // Continue to check for __PURE__ or __NO_SIDE_EFFECTS__ after #
            }
            b'w' => {
                // Check for webpack comments
                if bytes[start..].starts_with(b"webpack")
                    && start + 7 < bytes.len()
                    && bytes[start + 7].is_ascii_uppercase()
                {
                    comment.content = CommentContent::Webpack;
                    return;
                }
                // Fall through to check for coverage ignore patterns
            }
            b't' => {
                // Check for turbopack comments
                if bytes[start..].starts_with(b"turbopack")
                    && start + 9 < bytes.len()
                    && bytes[start + 9].is_ascii_uppercase()
                {
                    comment.content = CommentContent::Turbopack;
                    return;
                }
                // Fall through to check for coverage ignore patterns
            }
            b'v' | b'c' | b'n' | b'i' => {
                // Check coverage ignore patterns: "v8 ignore", "c8 ignore", "node:coverage", "istanbul ignore"
                let rest = &bytes[start..];
                if rest.starts_with(b"v8 ignore")
                    || rest.starts_with(b"c8 ignore")
                    || rest.starts_with(b"node:coverage")
                    || rest.starts_with(b"istanbul ignore")
                {
                    comment.content = if is_coverage_ignore_file(rest) {
                        CommentContent::CoverageIgnoreFile
                    } else {
                        CommentContent::CoverageIgnore
                    };
                    return;
                }
                // Fall through to check license/preserve
            }
            _ => {
                // Check for license/preserve comments in remaining cases
                if contains_license_or_preserve_comment(s) {
                    comment.content = CommentContent::Legal;
                }
                return;
            }
        }

        // Check for __PURE__ or __NO_SIDE_EFFECTS__ after @ or #
        if start < bytes.len() && bytes[start..].starts_with(b"__") {
            let rest = &bytes[start + 2..];
            if rest.starts_with(b"PURE__") {
                comment.content = CommentContent::PureNotApplied;
                return;
            } else if rest.starts_with(b"NO_SIDE_EFFECTS__") {
                comment.content = CommentContent::NoSideEffectsNotApplied;
                return;
            }
        }

        // Fallback: check for @license or @preserve anywhere in the comment
        // This handles cases like /* @foo @preserve */ where the first @ doesn't match known patterns
        if contains_license_or_preserve_comment(s) {
            comment.content = CommentContent::Legal;
        }
    }
}

#[inline]
fn is_property_key_annotation(source: &str) -> bool {
    matches!(source.trim().strip_prefix(['@', '#']), Some("__KEY__"))
}

#[inline(always)]
fn contains_license_or_preserve_comment(s: &str) -> bool {
    const LICENSE_LEN: usize = b"@license".len();
    const PRESERVE_LEN: usize = b"@preserve".len();
    let hay = s.as_bytes();

    if hay.len() < LICENSE_LEN {
        return false;
    }

    let search_len = hay.len() - LICENSE_LEN + 1;

    for i in memchr_iter(b'@', &hay[..search_len]) {
        debug_assert!(i < search_len);
        debug_assert!(hay.len() - i >= LICENSE_LEN);
        // SAFETY: `search_len` only includes candidate starts with at least `LICENSE_LEN` bytes
        // remaining, so `i + 1` and the full `@license` range are in bounds.
        match unsafe { hay.get_unchecked(i + 1) } {
            // spellchecker:off
            b'l'
                // SAFETY: The candidate bound proves the full `@license` range is in bounds.
                if unsafe { hay.get_unchecked(i + 2..i + LICENSE_LEN) } == b"icense" =>
            {
                return true;
            }
            b'p' if hay.len() - i >= PRESERVE_LEN
                    // SAFETY: The preceding guard proves the full `@preserve` range is in bounds.
                    && unsafe { hay.get_unchecked(i + 2..i + PRESERVE_LEN) } == b"reserve" =>
            {
                return true;
            }
            // spellchecker:on
            _ => {}
        }
    }

    false
}

fn is_coverage_ignore_file(source: &[u8]) -> bool {
    fn starts_with_directive(source: &[u8], directive: &[u8]) -> bool {
        source
            .strip_prefix(directive)
            .is_some_and(|rest| rest.first().is_none_or(u8::is_ascii_whitespace))
    }

    starts_with_directive(source, b"v8 ignore file")
        || starts_with_directive(source, b"istanbul ignore file")
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/parser/src/lexer/trivia_builder/test.rs"]
#[cfg(test)]
mod test;
