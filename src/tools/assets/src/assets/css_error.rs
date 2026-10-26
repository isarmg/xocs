//! Own borrowed CSS diagnostics while preserving their original kinds and locations.
use style_compiler::style_optimizer::error::{Error as CssError, ParserError};

pub(super) fn owned_css_error(error: CssError<ParserError<'_>>) -> CssError<ParserError<'static>> {
    use style_compiler::style_optimizer::error::ParserError::*;
    let kind = match error.kind {
        AtRuleBodyInvalid => AtRuleBodyInvalid,
        AtRulePreludeInvalid => AtRulePreludeInvalid,
        AtRuleInvalid(name) => AtRuleInvalid(name.to_string().into()),
        EndOfInput => EndOfInput,
        InvalidDeclaration => InvalidDeclaration,
        InvalidMediaQuery => InvalidMediaQuery,
        EmptyBracketInCondition => EmptyBracketInCondition,
        InvalidNesting => InvalidNesting,
        DeprecatedNestRule => DeprecatedNestRule,
        DeprecatedCssModulesValueRule => DeprecatedCssModulesValueRule,
        InvalidPageSelector => InvalidPageSelector,
        InvalidValue => InvalidValue,
        QualifiedRuleInvalid => QualifiedRuleInvalid,
        SelectorError(error) => SelectorError(owned_selector_error(error)),
        UnexpectedImportRule => UnexpectedImportRule,
        UnexpectedNamespaceRule => UnexpectedNamespaceRule,
        UnexpectedToken(token) => UnexpectedToken(owned_css_token(token)),
        MaximumNestingDepth => MaximumNestingDepth,
    };
    CssError {
        kind,
        loc: error.loc,
    }
}

fn owned_selector_error(
    error: style_compiler::style_optimizer::error::SelectorError<'_>,
) -> style_compiler::style_optimizer::error::SelectorError<'static> {
    use style_compiler::style_optimizer::error::SelectorError::*;
    match error {
        BadValueInAttr(token) => BadValueInAttr(owned_css_token(token)),
        ClassNeedsIdent(token) => ClassNeedsIdent(owned_css_token(token)),
        DanglingCombinator => DanglingCombinator,
        EmptySelector => EmptySelector,
        ExpectedBarInAttr(token) => ExpectedBarInAttr(owned_css_token(token)),
        ExpectedNamespace(name) => ExpectedNamespace(name.to_string().into()),
        ExplicitNamespaceUnexpectedToken(token) => {
            ExplicitNamespaceUnexpectedToken(owned_css_token(token))
        }
        InvalidPseudoClassAfterPseudoElement => InvalidPseudoClassAfterPseudoElement,
        InvalidPseudoClassAfterWebKitScrollbar => InvalidPseudoClassAfterWebKitScrollbar,
        InvalidPseudoClassBeforeWebKitScrollbar => InvalidPseudoClassBeforeWebKitScrollbar,
        InvalidQualNameInAttr(token) => InvalidQualNameInAttr(owned_css_token(token)),
        InvalidState => InvalidState,
        MissingNestingPrefix => MissingNestingPrefix,
        MissingNestingSelector => MissingNestingSelector,
        NoQualifiedNameInAttributeSelector(token) => {
            NoQualifiedNameInAttributeSelector(owned_css_token(token))
        }
        PseudoElementExpectedIdent(token) => PseudoElementExpectedIdent(owned_css_token(token)),
        UnexpectedIdent(name) => UnexpectedIdent(name.to_string().into()),
        UnexpectedTokenInAttributeSelector(token) => {
            UnexpectedTokenInAttributeSelector(owned_css_token(token))
        }
        UnsupportedPseudoClass(name) => UnsupportedPseudoClass(name.to_string().into()),
        UnsupportedPseudoElement(name) => UnsupportedPseudoElement(name.to_string().into()),
        AmbiguousCssModuleClass(name) => AmbiguousCssModuleClass(name.to_string().into()),
        UnexpectedSelectorAfterPseudoElement(token) => {
            UnexpectedSelectorAfterPseudoElement(owned_css_token(token))
        }
    }
}

fn owned_css_token(
    token: style_compiler::style_optimizer::properties::custom::Token<'_>,
) -> style_compiler::style_optimizer::properties::custom::Token<'static> {
    use style_compiler::style_optimizer::properties::custom::Token::*;
    match token {
        Ident(value) => Ident(value.to_string().into()),
        AtKeyword(value) => AtKeyword(value.to_string().into()),
        Hash(value) => Hash(value.to_string().into()),
        IDHash(value) => IDHash(value.to_string().into()),
        String(value) => String(value.to_string().into()),
        UnquotedUrl(value) => UnquotedUrl(value.to_string().into()),
        Delim(value) => Delim(value),
        Number {
            has_sign,
            value,
            int_value,
        } => Number {
            has_sign,
            value,
            int_value,
        },
        Percentage {
            has_sign,
            unit_value,
            int_value,
        } => Percentage {
            has_sign,
            unit_value,
            int_value,
        },
        Dimension {
            has_sign,
            value,
            int_value,
            unit,
        } => Dimension {
            has_sign,
            value,
            int_value,
            unit: unit.to_string().into(),
        },
        WhiteSpace(value) => WhiteSpace(value.to_string().into()),
        Comment(value) => Comment(value.to_string().into()),
        Colon => Colon,
        Semicolon => Semicolon,
        Comma => Comma,
        IncludeMatch => IncludeMatch,
        DashMatch => DashMatch,
        PrefixMatch => PrefixMatch,
        SuffixMatch => SuffixMatch,
        SubstringMatch => SubstringMatch,
        CDO => CDO,
        CDC => CDC,
        Function(value) => Function(value.to_string().into()),
        ParenthesisBlock => ParenthesisBlock,
        SquareBracketBlock => SquareBracketBlock,
        CurlyBracketBlock => CurlyBracketBlock,
        BadUrl(value) => BadUrl(value.to_string().into()),
        BadString(value) => BadString(value.to_string().into()),
        CloseParenthesis => CloseParenthesis,
        CloseSquareBracket => CloseSquareBracket,
        CloseCurlyBracket => CloseCurlyBracket,
    }
}
