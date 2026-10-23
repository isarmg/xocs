//! Oxc Parser for JavaScript and TypeScript
//!
//! Oxc's [`Parser`] has full support for
//! - The latest stable ECMAScript syntax
//! - TypeScript
//! - JSX and TSX
//! - [Stage 3 Decorators](https://github.com/tc39/proposal-decorator-metadata)
//!
//! # Usage
//!
//! The parser has a minimal API with three inputs (a [memory arena](crate::syntax_arena::Allocator), a
//! source string, and a [`SourceType`]) and one return struct (a [ParserReturn]).
//!
//! ```rust,ignore
//! let parser_return = Parser::new(&allocator, &source_text, source_type).parse();
//! ```
//!
//! # Abstract Syntax Tree (AST)
//! Oxc's AST is located in a separate [`crate::syntax_tree`] crate. You can find type definitions for AST
//! nodes [here][`crate::syntax_tree::ast`].
//!
//! # Performance
//!
//! The following optimization techniques are used:
//! * AST is allocated in a memory arena ([oxc_allocator](https://docs.rs/oxc_allocator)) for fast AST drop
//! * [`crate::source_location::Span`] offsets uses `u32` instead of `usize`
//! * Scope binding, symbol resolution and complicated syntax errors are not done in the parser,
//! they are delegated to the [semantic analyzer](https://docs.rs/oxc_semantic)
//!
//! <div class="warning">
//! Because [`crate::source_location::Span`] uses `u32` instead of `usize`, Oxc can only parse files up
//! to 4 GiB in size. This shouldn't be a limitation in almost all cases.
//! </div>
//!
//! # Examples
//!
//! <https://github.com/oxc-project/oxc/blob/main/crates/oxc_parser/examples/parser.rs>
//!
//! ```rust,ignore
#![doc = include_str!("../examples/parser.rs")]
//! ```
//!
//! ### Parsing TSX
//! ```rust,ignore
#![doc = include_str!("../examples/parser_tsx.rs")]
//! ```
//!
//! # Visitor
//!
//! See [`Visit`](http://docs.rs/oxc_ast_visit) and [`VisitMut`](http://docs.rs/oxc_ast_visit).
//!
//! # Visiting without a visitor
//!
//! For ad-hoc tasks, the semantic analyzer can be used to get a parent pointing tree with untyped nodes,
//! the nodes can be iterated through a sequential loop.
//!
//! ```rust,ignore
//! for node in semantic.nodes().iter() {
//!     match node.kind() {
//!         // check node
//!     }
//! }
//! ```
//!
//! See [full linter example](https://github.com/Boshen/oxc/blob/ab2ef4f89ba3ca50c68abb2ca43e36b7793f3673/crates/oxc_linter/examples/linter.rs#L38-L39)

use std::any::Any;

pub mod config;
mod context;
mod cursor;
mod error_handler;
mod modifiers;
mod module_record;
mod state;

mod js;
mod jsx;
mod ts;

mod diagnostics;

// Expose lexer only in benchmarks
#[cfg(not(any()))]
mod lexer;
#[cfg(any())]
#[doc(hidden)]
pub mod lexer;

use crate::syntax_arena::{Allocator, ArenaBox, ArenaVec, Dummy, GetAllocator};
use crate::syntax_tree::{
    ast::{Expression, Program, Statement},
    builder::{AstBuilder, GetAstBuilder},
};
use crate::compiler_errors::Diagnostics;
use crate::source_location::{SourceType, Span};
use crate::syntax_types::module_record::ModuleRecord;

pub use crate::parser::lexer::{Kind, Token};
use crate::parser::{
    config::{
        LexerConfig, NoTokensParserConfig, ParserConfig, RuntimeParserConfig, TokensParserConfig,
    },
    context::{Context, StatementContext},
    diagnostics::ParserDiagnostic,
    error_handler::FatalError,
    lexer::Lexer,
    module_record::ModuleRecordBuilder,
    state::ParserState,
};

/// Maximum length of source which can be parsed (in bytes).
/// ~4 GiB on 64-bit systems, ~2 GiB on 32-bit systems.
//
// Length is constrained by 2 factors:
// 1. `Span`'s `start` and `end` are `u32`s, which limits length to `u32::MAX` bytes.
// 2. Rust's allocator APIs limit allocations to `isize::MAX`.
//    https://doc.rust-lang.org/std/alloc/struct.Layout.html#method.from_size_align
//
// On 64-bit systems, the limit is 256 bytes below `u32::MAX` rather than `u32::MAX` itself:
//
// 1. `oxc_lexer` will require source text to be followed by 64 bytes of padding.
//    This means *padded* length can fit in a `u32`.
//    256 instead of 64 to leave headroom, in case the padding requirement expands in future.
// 2. Counts of tokens, comments and errors are all bounded by source length plus a small constant,
//    so they can be stored as `u32`s without any possibility of overflow.
// 3. No real `Span` can have `start` or `end` of `u32::MAX`, so such a `Span` can be used as a sentinel.
//    `oxc_transformer`'s styled-components plugin relies on this.
pub(crate) const MAX_LEN: usize = if size_of::<usize>() >= 8 {
    // 64-bit systems
    u32::MAX as usize - 256
} else {
    // 32-bit or 16-bit systems
    isize::MAX as usize
};

/// Return value of [`Parser::parse`] consisting of AST, errors and comments
///
/// ## AST Validity
///
/// [`program`] will always contain a structurally valid AST, even if there are syntax errors.
/// However, the AST may be semantically invalid. To ensure a valid AST,
/// 1. Check that [`diagnostics`] is empty
/// 2. Run semantic analysis with [syntax error checking
///    enabled](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html#method.with_check_syntax_error)
///
/// ## Errors
///
/// Oxc's [`Parser`] is able to recover from some syntax errors and continue parsing.
/// When this happens:
///
/// 1. [`diagnostics`] will be non-empty
/// 2. [`program`] will contain a full AST
/// 3. [`fatal_error`] will be false
///
/// When the parser cannot recover, it will abort and terminate parsing early.
/// [`program`] will be empty and [`fatal_error`] will be `true`.
///
/// [`program`]: ParserReturn::program
/// [`diagnostics`]: ParserReturn::diagnostics
/// [`fatal_error`]: ParserReturn::fatal_error
#[non_exhaustive]
pub struct ParserReturn<'a> {
    /// The parsed AST.
    ///
    /// Will be empty (e.g. no statements, directives, etc) if the parser encountered a fatal error.
    ///
    /// ## Validity
    /// It is possible for the AST to be present and semantically invalid. This will happen if
    /// 1. The [`Parser`] encounters a recoverable syntax error
    /// 2. The logic for checking the violation is in the semantic analyzer
    ///
    /// To ensure a valid AST, check that [`diagnostics`](ParserReturn::diagnostics) is empty. Then, run
    /// semantic analysis with syntax error checking enabled.
    pub program: Program<'a>,

    /// See <https://tc39.es/ecma262/#sec-abstract-module-records>
    pub module_record: ModuleRecord<'a>,

    /// Syntax errors encountered while parsing.
    ///
    /// This list is not comprehensive. Oxc offloads more-expensive checks to [semantic
    /// analysis](https://docs.rs/oxc_semantic), which can be enabled using
    /// [`SemanticBuilder::with_check_syntax_error`](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html#method.with_check_syntax_error).
    pub diagnostics: Diagnostics,

    /// Irregular whitespaces for `Oxlint`
    pub irregular_whitespaces: Box<[Span]>,

    /// Lexed tokens in source order.
    ///
    /// Tokens are only collected when tokens are enabled in [`ParserConfig`].
    pub tokens: ArenaVec<'a, Token>,

    /// Whether the parser encountered a fatal error and terminated early.
    ///
    /// If `false`, either parsing was successful, or parsing was able to recover from a syntax error.
    /// [`diagnostics`] may still contain errors, indicating that the program is not syntactically valid,
    /// but parser was able to recover enough to produce an AST.
    ///
    /// If `true`, parser was unable to parse the source.
    /// [`program`] will be empty and [`diagnostics`] will contain at least one error.
    ///
    /// [`program`]: ParserReturn::program
    /// [`diagnostics`]: ParserReturn::diagnostics
    pub fatal_error: bool,

    /// Whether the file is [flow](https://flow.org).
    pub is_flow_language: bool,
}

/// Parse options
///
/// You may provide options to the [`Parser`] using [`Parser::with_options`].
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Whether to parse regular expressions or not.
    ///
    /// Default: `false`
    #[cfg(all())]
    pub parse_regular_expression: bool,

    /// Allow [`return`] statements outside of functions.
    ///
    /// By default, a return statement at the top level raises an error (`false`).
    ///
    /// Default: `false`
    ///
    /// [`return`]: crate::syntax_tree::ast::ReturnStatement
    pub allow_return_outside_function: bool,

    /// Emit [`ParenthesizedExpression`]s and [`TSParenthesizedType`] in AST.
    ///
    /// If this option is `true`, parenthesized expressions are represented by
    /// (non-standard) [`ParenthesizedExpression`] and [`TSParenthesizedType`] nodes
    /// that have a single `expression` property containing the expression inside parentheses.
    ///
    /// Default: `true`
    ///
    /// [`ParenthesizedExpression`]: crate::syntax_tree::ast::ParenthesizedExpression
    /// [`TSParenthesizedType`]: crate::syntax_tree::ast::TSParenthesizedType
    pub preserve_parens: bool,

    /// Allow V8 runtime calls in the AST.
    /// See: [V8's Parser::ParseV8Intrinsic](https://chromium.googlesource.com/v8/v8/+/35a14c75e397302655d7b3fbe648f9490ae84b7d/src/parsing/parser.cc#4811).
    ///
    /// Default: `false`
    ///
    /// [`V8IntrinsicExpression`]: crate::syntax_tree::ast::V8IntrinsicExpression
    pub allow_v8_intrinsics: bool,

    /// Precompute hashes for identifier names (`Ident`s) in the AST.
    ///
    /// Precomputed hashes speed up `Ident`-keyed hash map operations in semantic analysis
    /// and later compilation stages, at a small cost to parsing speed.
    ///
    /// Only disable this for parse-only pipelines that never rely on `Ident` hashing, such as
    /// parse + serialize or formatting. Unhashed `Ident`s do not compare equal to hashed ones,
    /// so semantic analysis and anything else relying on `Ident` hashing must not run on an AST
    /// parsed with this option disabled.
    ///
    /// Default: `true`
    pub enable_ident_hashes: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            #[cfg(all())]
            parse_regular_expression: false,
            allow_return_outside_function: false,
            preserve_parens: true,
            allow_v8_intrinsics: false,
            enable_ident_hashes: true,
        }
    }
}

/// Recursive Descent Parser for ECMAScript and TypeScript
///
/// See [`Parser::parse`] for entry function.
pub struct Parser<'a, C: ParserConfig = NoTokensParserConfig> {
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    options: ParseOptions,
    config: C,
}

impl<'a> Parser<'a> {
    /// Create a new [`Parser`]
    ///
    /// # Parameters
    /// - `allocator`: [Memory arena](crate::syntax_arena::Allocator) for allocating AST nodes
    /// - `source_text`: Source code to parse
    /// - `source_type`: Source type (e.g. JavaScript, TypeScript, JSX, ESM Module, Script)
    pub fn new(allocator: &'a Allocator, source_text: &'a str, source_type: SourceType) -> Self {
        let options = ParseOptions::default();
        Self { allocator, source_text, source_type, options, config: NoTokensParserConfig }
    }
}

impl<'a, C: ParserConfig> Parser<'a, C> {
    /// Set parse options
    #[must_use]
    pub fn with_options(mut self, options: ParseOptions) -> Self {
        self.options = options;
        self
    }

    /// Set parser config.
    ///
    /// See [`ParserConfig`] for more details.
    #[must_use]
    pub fn with_config<Config: ParserConfig>(self, config: Config) -> Parser<'a, Config> {
        Parser {
            allocator: self.allocator,
            source_text: self.source_text,
            source_type: self.source_type,
            options: self.options,
            config,
        }
    }
}

mod parser_parse {
    use super::*;

    /// `UniquePromise` is a way to use the type system to enforce the invariant that only
    /// a single `ParserImpl`, `Lexer` and `lexer::Source` can exist at any time on a thread.
    /// This constraint is required to guarantee the soundness of some methods of these types
    /// e.g. `Source::set_position`.
    ///
    /// `ParserImpl::new`, `Lexer::new` and `lexer::Source::new` all require a `UniquePromise`
    /// to be provided to them. `UniquePromise::new` is not visible outside this module, so only
    /// `Parser::parse` can create one, and it only calls `ParserImpl::new` once.
    /// This enforces the invariant throughout the entire parser.
    ///
    /// `UniquePromise` is a zero-sized type and has no runtime cost. It's purely for the type-checker.
    ///
    /// `UniquePromise::new_for_tests_and_benchmarks` is a backdoor for tests/benchmarks, so they can
    /// create a `ParserImpl` or `Lexer`, and manipulate it directly, for testing/benchmarking purposes.
    pub struct UniquePromise(());

    impl UniquePromise {
        #[inline]
        fn new() -> Self {
            Self(())
        }

        /// Backdoor for tests/benchmarks to create a `UniquePromise` (see above).
        /// This function must NOT be exposed outside of tests and benchmarks,
        /// as it allows circumventing safety invariants of the parser.
        #[cfg(any(test, any()))]
        pub fn new_for_tests_and_benchmarks() -> Self {
            Self(())
        }
    }

    impl<'a, C: ParserConfig> Parser<'a, C> {
        /// Main entry point
        ///
        /// Returns an empty `Program` on unrecoverable error,
        /// Recoverable errors are stored inside `errors`.
        ///
        /// See the [module-level documentation](crate) for examples and more information.
        //
        // # Implementation note
        //
        // Dispatches via `Any` to a non-generic helper for each known `ParserConfig`.
        // The dispatch keeps the parser body emitted exactly once in `oxc_parser`'s rlib,
        // so consuming crates don't each get a private copy. The `Any::is` / `downcast_ref`
        // calls const-fold when `C` is concrete (the trait object is built from a known
        // concrete type at every monomorphization site, so LLVM devirtualises the vtable
        // call to `Any::type_id` and folds the comparison), leaving each monomorphization
        // as a single call into the matching helper.
        pub fn parse(self) -> ParserReturn<'a> {
            let config: &dyn Any = &self.config;
            if config.is::<NoTokensParserConfig>() {
                parse_with_no_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                )
            } else if config.is::<TokensParserConfig>() {
                parse_with_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                )
            } else if let Some(&config) = config.downcast_ref::<RuntimeParserConfig>() {
                parse_with_runtime_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                    config,
                )
            } else {
                // User-defined `ParserConfig`. Generic codegen here, monomorphized per consuming crate.
                // Users using custom configs would need to perform the monomorphization themselves.
                ParserImpl::<C>::new(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                    self.config,
                    UniquePromise::new(),
                )
                .parse()
            }
        }

        /// Parse a single [`Expression`].
        ///
        /// # Example
        ///
        /// ```rust
        /// use crate::syntax_arena::Allocator;
        /// use crate::syntax_tree::ast::Expression;
        /// use oxc_parser::Parser;
        /// use crate::source_location::SourceType;
        ///
        /// let src = "1 + 2";
        /// let allocator = Allocator::new();
        /// let source_type = SourceType::default();
        ///
        /// let expr: Expression<'_> = Parser::new(&allocator, src, source_type).parse_expression().unwrap();
        /// ```
        ///
        /// # Errors
        /// If the source code being parsed has syntax errors.
        //
        // # Implementation note
        // Dispatches via `Any`, same as `parse` does.
        pub fn parse_expression(self) -> Result<Expression<'a>, Diagnostics> {
            let config: &dyn Any = &self.config;
            if config.is::<NoTokensParserConfig>() {
                parse_expression_with_no_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                )
            } else if config.is::<TokensParserConfig>() {
                parse_expression_with_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                )
            } else if let Some(&config) = config.downcast_ref::<RuntimeParserConfig>() {
                parse_expression_with_runtime_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                    config,
                )
            } else {
                ParserImpl::<C>::new(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.options,
                    self.config,
                    UniquePromise::new(),
                )
                .parse_expression()
            }
        }
    }

    // ===========================================================================
    // Non-generic parse helpers, one per known `ParserConfig`.
    //
    // The parser is generic over `C: ParserConfig`. By default Rust monomorphizes
    // generic functions per consuming crate (the legacy mangled name encodes the
    // instantiating crate's disambiguator, and `share-generics` is off at
    // `opt-level >= 2`). For a parser that's pulled in by ~15 crates in a real
    // workspace, that means ~15 private copies of every parser method in the
    // final cdylib — none of which can be deduped by COMDAT (different names)
    // or fat LTO (slightly different inlining contexts).
    //
    // To avoid that, `Parser<C>::parse` and `Parser<C>::parse_expression` dispatch
    // via `Any` to one of the helpers below for the three known configs.
    // Each helper is non-generic, so it's emitted exactly once in `oxc_parser`'s
    // rlib and shared by all consumers. The `Any::is` / `downcast_ref` checks fold
    // at compile time when `C` is concrete, so each monomorphization of the dispatch
    // shrinks to a single call into the matching helper.
    //
    // The helpers are `#[inline(never)]` to prevent fat LTO from re-inlining the
    // parser body across the rlib boundary, which would defeat the purpose.
    //
    // For user-defined `ParserConfig` impls (rare), the dispatch falls through
    // to a generic body that monomorphizes per consuming crate. That's the same
    // cost the parser had without this pattern; we just keep that cost contained
    // to the rare custom-config case.
    // ===========================================================================

    #[inline(never)]
    fn parse_with_no_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
    ) -> ParserReturn<'a> {
        ParserImpl::<NoTokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            options,
            NoTokensParserConfig,
            UniquePromise::new(),
        )
        .parse()
    }

    #[inline(never)]
    fn parse_with_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
    ) -> ParserReturn<'a> {
        ParserImpl::<TokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            options,
            TokensParserConfig,
            UniquePromise::new(),
        )
        .parse()
    }

    #[inline(never)]
    fn parse_with_runtime_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
        config: RuntimeParserConfig,
    ) -> ParserReturn<'a> {
        ParserImpl::<RuntimeParserConfig>::new(
            allocator,
            source_text,
            source_type,
            options,
            config,
            UniquePromise::new(),
        )
        .parse()
    }

    #[inline(never)]
    fn parse_expression_with_no_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
    ) -> Result<Expression<'a>, Diagnostics> {
        ParserImpl::<NoTokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            options,
            NoTokensParserConfig,
            UniquePromise::new(),
        )
        .parse_expression()
    }

    #[inline(never)]
    fn parse_expression_with_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
    ) -> Result<Expression<'a>, Diagnostics> {
        ParserImpl::<TokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            options,
            TokensParserConfig,
            UniquePromise::new(),
        )
        .parse_expression()
    }

    #[inline(never)]
    fn parse_expression_with_runtime_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
        config: RuntimeParserConfig,
    ) -> Result<Expression<'a>, Diagnostics> {
        ParserImpl::<RuntimeParserConfig>::new(
            allocator,
            source_text,
            source_type,
            options,
            config,
            UniquePromise::new(),
        )
        .parse_expression()
    }
}
use parser_parse::UniquePromise;

/// Implementation of parser.
/// `Parser` is just a public wrapper, the guts of the implementation is in this type.
struct ParserImpl<'a, C: ParserConfig> {
    /// Options
    options: ParseOptions,

    pub(crate) lexer: Lexer<'a, C::LexerConfig>,

    /// SourceType: JavaScript or TypeScript, Script or Module, jsx support?
    source_type: SourceType,

    /// Source Code
    source_text: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    ///
    /// Stored in deferred (unmaterialized) form and materialized once at parse exit.
    errors: Vec<ParserDiagnostic<'a>>,

    /// Errors that are only valid if the file is determined to be a Script (not a Module).
    /// For `ModuleKind::Unambiguous`, we defer ESM-only errors (like top-level await)
    /// until we know whether the file is ESM or Script.
    /// If resolved to Module → discard these errors.
    /// If resolved to Script → emit these errors.
    deferred_script_errors: Vec<ParserDiagnostic<'a>>,

    fatal_error: Option<FatalError<'a>>,

    /// The current parsing token
    token: Token,

    /// The end range of the previous token
    prev_token_end: u32,

    /// Parser state
    state: ParserState<'a>,

    /// Parsing context
    ctx: Context,

    /// Ast builder for creating AST nodes
    ast: AstBuilder<'a>,

    /// Module Record Builder
    module_record_builder: ModuleRecordBuilder<'a>,

    /// Precomputed typescript detection
    is_ts: bool,
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    /// Create a new `ParserImpl`.
    ///
    /// Requiring a `UniquePromise` to be provided guarantees only 1 `ParserImpl` can exist
    /// on a single thread at one time.
    #[inline]
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
        config: C,
        unique: UniquePromise,
    ) -> Self {
        Self {
            options,
            lexer: Lexer::new(allocator, source_text, source_type, config.lexer_config(), unique),
            source_type,
            source_text,
            errors: vec![],
            deferred_script_errors: vec![],
            fatal_error: None,
            token: Token::default(),
            prev_token_end: 0,
            state: ParserState::new(),
            ctx: Self::default_context(source_type, options),
            ast: AstBuilder::new(allocator),
            module_record_builder: ModuleRecordBuilder::new(allocator, source_type),
            is_ts: source_type.is_typescript(),
        }
    }

    /// Main entry point
    ///
    /// Returns an empty `Program` on unrecoverable error,
    /// Recoverable errors are stored inside `errors`.
    #[inline]
    pub fn parse(mut self) -> ParserReturn<'a> {
        let mut program = self.parse_program();
        let mut has_fatal_error = false;

        if let Some(fatal_error) = self.fatal_error.take() {
            has_fatal_error = true;
            self.errors.truncate(fatal_error.errors_len);
            if !self.lexer.errors.is_empty() && self.cur_kind().is_eof() {
                // Noop
            } else {
                self.error(fatal_error.error);
            }

            program = Program::dummy(self.allocator());
            program.source_type = self.source_type;
            program.source_text = self.source_text;
        }

        self.check_unfinished_errors();

        if let Some(overlong_error) = self.overlong_error() {
            has_fatal_error = true;
            self.lexer.errors.clear();
            self.errors.clear();
            self.error(overlong_error);
        }

        let mut is_flow_language = false;
        let mut errors = Diagnostics::new();
        // only check for `@flow` if the file failed to parse.
        if (!self.lexer.errors.is_empty() || !self.errors.is_empty())
            && let Some(error) = self.flow_error()
        {
            is_flow_language = true;
            errors.push(error.into_diagnostic());
        }
        let (module_record, mut module_record_errors) = self.module_record_builder.build();
        if errors.len() != 1 {
            errors
                .reserve(self.lexer.errors.len() + self.errors.len() + module_record_errors.len());
            errors.extend(self.lexer.errors.drain(..).map(ParserDiagnostic::into_diagnostic));
            errors.extend(self.errors.drain(..).map(ParserDiagnostic::into_diagnostic));
            errors.append(&mut module_record_errors);
        }
        let irregular_whitespaces =
            std::mem::take(&mut self.lexer.trivia_builder.irregular_whitespaces).into_boxed_slice();

        let source_type = program.source_type;
        if source_type.is_unambiguous() {
            if module_record.has_module_syntax {
                // Resolved to Module - discard deferred script errors (TLA is valid in ESM)
                // but emit deferred module errors (HTML comments are invalid in ESM)
                program.source_type = source_type.with_module(true);
                errors.extend(
                    self.lexer
                        .deferred_module_errors
                        .drain(..)
                        .map(ParserDiagnostic::into_diagnostic),
                );
            } else {
                // Resolved to Script - emit deferred script errors
                // discard deferred module errors (HTML comments are valid in scripts)
                program.source_type = source_type.with_script(true);
                errors.extend(
                    self.deferred_script_errors.into_iter().map(ParserDiagnostic::into_diagnostic),
                );
            }
        }

        let tokens = if has_fatal_error {
            ArenaVec::new_in(&self.ast)
        } else {
            self.lexer.finalize_tokens()
        };

        program.comments = self.lexer.trivia_builder.comments;

        ParserReturn {
            program,
            module_record,
            diagnostics: errors,
            irregular_whitespaces,
            tokens,
            fatal_error: has_fatal_error,
            is_flow_language,
        }
    }

    pub fn parse_expression(mut self) -> Result<Expression<'a>, Diagnostics> {
        // initialize cur_token and prev_token by moving onto the first token
        self.bump_any();
        let expr = self.parse_expr();
        if !self.at(Kind::Eof) {
            self.set_unexpected();
        }
        if let Some(FatalError { error, .. }) = self.fatal_error.take() {
            return Err(error.into_diagnostic().into());
        }
        self.check_unfinished_errors();
        let errors = self
            .lexer
            .errors
            .into_iter()
            .chain(self.errors)
            .map(ParserDiagnostic::into_diagnostic)
            .collect::<Diagnostics>();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(expr)
    }

    #[expect(clippy::cast_possible_truncation)]
    fn parse_program(&mut self) -> Program<'a> {
        // Initialize by moving onto the first token.
        // Checks for hashbang comment.
        self.token = self.lexer.first_token();

        let hashbang = self.parse_hashbang();
        self.ctx |= Context::TopLevel;
        let (directives, mut statements) =
            self.parse_directives_and_statements(/* in_ts_namespace_body */ false);

        // In unambiguous mode, if ESM syntax was detected (import/export/import.meta),
        // we need to reparse statements that were originally parsed with `await` as identifier.
        // TypeScript's behavior: initially parse `await /x/` as division, then reparse as
        // await expression with regex when ESM is detected.
        // Preserve a fatal error from the initial parse instead of rewinding past it.
        if self.fatal_error.is_none()
            && self.source_type.is_unambiguous()
            && self.module_record_builder.has_module_syntax()
            && !self.state.potential_await_reparse.is_empty()
        {
            self.reparse_potential_top_level_awaits(&mut statements);
        }

        let span = Span::new(0, self.source_text.len() as u32);
        Program::new(
            span,
            self.source_type,
            self.source_text,
            // Populated at the end of `parse` after `flow_error` has read from `trivia_builder.comments`
            [],
            hashbang,
            directives,
            statements,
            self,
        )
    }

    /// Reparse statements that may contain top-level await expressions.
    ///
    /// In unambiguous mode, statements like `await /x/u` are initially parsed as
    /// `await / x / u` (identifier with divisions). If ESM syntax is detected,
    /// we need to reparse them with the await context enabled.
    fn reparse_potential_top_level_awaits(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        let original_tokens =
            if self.lexer.config.tokens() { Some(self.lexer.take_tokens()) } else { None };
        let original = original_tokens.as_ref().map_or(&[][..], |tokens| tokens.as_slice());
        let mut original_index = 0;
        let mut replacements = ArenaVec::new_in(self);

        let checkpoints = std::mem::take(&mut self.state.potential_await_reparse);
        // Ranges refer to the original tokens and to the flat replacement buffer
        let mut edits = ArenaVec::with_capacity_in(
            if self.lexer.config.tokens() { checkpoints.len() } else { 0 },
            self,
        );
        for (stmt_index, checkpoint) in checkpoints {
            self.rewind(checkpoint);
            let replacement_start = replacements.len();

            if self.lexer.config.tokens() {
                original_index += original[original_index..]
                    .iter()
                    .position(|token| token.start() >= self.token.start())
                    .unwrap_or(original.len() - original_index);
                // The checkpoint's current token has already been lexed
                replacements.push(self.token);
                self.lexer.set_tokens(std::mem::replace(&mut replacements, ArenaVec::new_in(self)));
            }
            let edit_start = original_index;

            // Parse the statement with await context enabled (TopLevel context is already set)
            let stmt = self.context_add(Context::Await, |p| {
                p.parse_statement_list_item(StatementContext::Program)
            });

            // Replace the statement if the index is valid
            if stmt_index < statements.len() {
                statements[stmt_index] = stmt;
            }

            if self.lexer.config.tokens() {
                replacements = self.lexer.take_tokens();
                // Exclude the next statement's token (or EOF)
                replacements.pop();
                // An earlier reparse can consume a later candidate, as in `await\nawait /x/u`.
                // Record overlapping source ranges only once.
                let end = replacements[..replacement_start].last().map_or(0, Token::end);
                let overlap_len =
                    replacements[replacement_start..].partition_point(|token| token.start() < end);
                if overlap_len != 0 {
                    replacements.copy_within(replacement_start + overlap_len.., replacement_start);
                    let len = replacements.len() - overlap_len;
                    replacements.truncate(len);
                }
                original_index += original[original_index..]
                    .iter()
                    .position(|token| token.start() >= self.token.start())
                    .unwrap_or(original.len() - original_index);
                edits.push((edit_start..original_index, replacement_start..replacements.len()));
            }
        }

        if let Some(mut tokens) = original_tokens {
            let mut read = 0;
            if edits.iter().all(|(range, replacement)| replacement.len() <= range.len()) {
                // Compact in place. Original indices remain valid because the write cursor
                // never overtakes the read cursor.
                let mut write = 0;
                for (range, replacement) in edits {
                    tokens.copy_within(read..range.start, write);
                    write += range.start - read;
                    let replacement = &replacements[replacement];
                    tokens[write..write + replacement.len()].copy_from_slice(replacement);
                    write += replacement.len();
                    read = range.end;
                }
                let tail_len = tokens.len() - read;
                tokens.copy_within(read.., write);
                tokens.truncate(write + tail_len);
                self.lexer.set_tokens(tokens);
            } else {
                // A growing replacement could overwrite unread tokens.
                // Use a separate output buffer, still applying all edits in one forward pass.
                let len = edits.iter().fold(tokens.len(), |len, (range, replacement)| {
                    len - range.len() + replacement.len()
                });
                let mut output = ArenaVec::with_capacity_in(len, self);
                for (range, replacement) in edits {
                    output.extend_from_slice(&tokens[read..range.start]);
                    output.extend_from_slice(&replacements[replacement]);
                    read = range.end;
                }
                output.extend_from_slice(&tokens[read..]);
                self.lexer.set_tokens(output);
            }
        }
    }

    fn default_context(source_type: SourceType, options: ParseOptions) -> Context {
        let mut ctx = Context::default().and_ambient(source_type.is_typescript_definition());
        if source_type.is_module() {
            // for [top-level-await](https://tc39.es/proposal-top-level-await/)
            ctx = ctx.and_await(true);
        }
        // CommonJS files are wrapped in a function, so return and `new.target`
        // are allowed at top-level
        if options.allow_return_outside_function || source_type.is_commonjs() {
            ctx = ctx.and_return(true);
        }
        if source_type.is_commonjs() {
            ctx = ctx.and_new_target(true);
        }
        ctx
    }

    /// Check for Flow declaration if the file cannot be parsed.
    /// The declaration must be [on the first line before any code](https://flow.org/en/docs/usage/#toc-prepare-your-code-for-flow)
    fn flow_error(&mut self) -> Option<ParserDiagnostic<'a>> {
        if !self.source_type.is_javascript() {
            return None;
        }
        let span = self.lexer.trivia_builder.comments.first()?.span;
        if span.source_text(self.source_text).contains("@flow") {
            self.errors.clear();
            Some(diagnostics::flow(span))
        } else {
            None
        }
    }

    fn check_unfinished_errors(&mut self) {
        use crate::source_location::GetSpan;
        // PropertyDefinition : cover_initialized_name
        // It is a Syntax Error if any source text is matched by this production.
        for expr in self.state.cover_initialized_name.values() {
            self.errors.push(diagnostics::cover_initialized_name(expr.span()));
        }
    }

    /// Check if source length exceeds MAX_LEN, if the file cannot be parsed.
    /// Original parsing error is not real - `Lexer::new` substituted "\0" as the source text.
    #[cold]
    fn overlong_error(&self) -> Option<ParserDiagnostic<'a>> {
        if self.source_text.len() > MAX_LEN {
            return Some(diagnostics::overlong_source());
        }
        None
    }

    #[inline]
    fn alloc<T>(&self, value: T) -> ArenaBox<'a, T> {
        ArenaBox::new_in(value, self)
    }
}

impl<'a, C: ParserConfig> GetAllocator<'a> for ParserImpl<'a, C> {
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.ast.allocator()
    }
}

impl<'a, C: ParserConfig> GetAstBuilder<'a> for ParserImpl<'a, C> {
    type Builder = AstBuilder<'a>;

    #[inline]
    fn builder(&self) -> &AstBuilder<'a> {
        &self.ast
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/parser/src/lib/test.rs"]
#[cfg(test)]
mod test;
