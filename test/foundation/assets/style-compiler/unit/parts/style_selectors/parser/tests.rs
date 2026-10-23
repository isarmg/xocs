
  use super::*;
  use crate::style_selectors::builder::SelectorFlags;
  use crate::style_selectors::parser;
  use crate::style_parser::{serialize_identifier, serialize_string, Parser as CssParser, ParserInput, ToCss};
  use std::collections::HashMap;
  use std::fmt;

  #[derive(Clone, Debug, Eq, PartialEq, Hash)]
  pub enum PseudoClass {
    Hover,
    Active,
    Lang(String),
  }

  #[derive(Clone, Debug, Eq, PartialEq, Hash)]
  pub enum PseudoElement {
    Before,
    After,
  }

  impl<'i> parser::PseudoElement<'i> for PseudoElement {
    type Impl = DummySelectorImpl;

    fn accepts_state_pseudo_classes(&self) -> bool {
      true
    }

    fn valid_after_slotted(&self) -> bool {
      true
    }
  }

  impl<'i> parser::NonTSPseudoClass<'i> for PseudoClass {
    type Impl = DummySelectorImpl;

    #[inline]
    fn is_active_or_hover(&self) -> bool {
      matches!(*self, PseudoClass::Active | PseudoClass::Hover)
    }

    #[inline]
    fn is_user_action_state(&self) -> bool {
      self.is_active_or_hover()
    }
  }

  impl ToCss for PseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
      W: fmt::Write,
    {
      match *self {
        PseudoClass::Hover => dest.write_str(":hover"),
        PseudoClass::Active => dest.write_str(":active"),
        PseudoClass::Lang(ref lang) => {
          dest.write_str(":lang(")?;
          serialize_identifier(lang, dest)?;
          dest.write_char(')')
        }
      }
    }
  }

  impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
      W: fmt::Write,
    {
      match *self {
        PseudoElement::Before => dest.write_str("::before"),
        PseudoElement::After => dest.write_str("::after"),
      }
    }
  }

  #[derive(Clone, Debug, PartialEq)]
  pub struct DummySelectorImpl;

  #[derive(Default)]
  pub struct DummyParser {
    default_ns: Option<DummyAtom>,
    ns_prefixes: HashMap<DummyAtom, DummyAtom>,
  }

  impl DummyParser {
    fn default_with_namespace(default_ns: DummyAtom) -> DummyParser {
      DummyParser {
        default_ns: Some(default_ns),
        ns_prefixes: Default::default(),
      }
    }
  }

  impl<'i> SelectorImpl<'i> for DummySelectorImpl {
    type ExtraMatchingData = ();
    type AttrValue = DummyAttrValue;
    type Identifier = DummyAtom;
    type LocalName = DummyAtom;
    type NamespaceUrl = DummyAtom;
    type NamespacePrefix = DummyAtom;
    type BorrowedLocalName = DummyAtom;
    type BorrowedNamespaceUrl = DummyAtom;
    type NonTSPseudoClass = PseudoClass;
    type PseudoElement = PseudoElement;
    type VendorPrefix = u8;
  }

  #[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
  pub struct DummyAttrValue(String);

  impl ToCss for DummyAttrValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
      W: fmt::Write,
    {
      serialize_string(&self.0, dest)
    }
  }

  impl AsRef<str> for DummyAttrValue {
    fn as_ref(&self) -> &str {
      self.0.as_ref()
    }
  }

  impl<'a> From<&'a str> for DummyAttrValue {
    fn from(string: &'a str) -> Self {
      Self(string.into())
    }
  }

  impl<'a> From<std::borrow::Cow<'a, str>> for DummyAttrValue {
    fn from(string: std::borrow::Cow<'a, str>) -> Self {
      Self(string.to_string())
    }
  }

  impl<'a> From<CowRcStr<'a>> for DummyAttrValue {
    fn from(string: CowRcStr<'a>) -> Self {
      Self(string.to_string())
    }
  }

  #[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
  pub struct DummyAtom(String);

  impl ToCss for DummyAtom {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
      W: fmt::Write,
    {
      serialize_identifier(&self.0, dest)
    }
  }

  impl From<String> for DummyAtom {
    fn from(string: String) -> Self {
      DummyAtom(string)
    }
  }

  impl<'a> From<&'a str> for DummyAtom {
    fn from(string: &'a str) -> Self {
      DummyAtom(string.into())
    }
  }

  impl<'a> From<CowRcStr<'a>> for DummyAtom {
    fn from(string: CowRcStr<'a>) -> Self {
      DummyAtom(string.to_string())
    }
  }

  impl AsRef<str> for DummyAtom {
    fn as_ref(&self) -> &str {
      self.0.as_ref()
    }
  }

  impl<'a> From<std::borrow::Cow<'a, str>> for DummyAtom {
    fn from(string: std::borrow::Cow<'a, str>) -> Self {
      Self(string.to_string())
    }
  }

  impl<'i> Parser<'i> for DummyParser {
    type Impl = DummySelectorImpl;
    type Error = SelectorParseErrorKind<'i>;

    fn parse_slotted(&self) -> bool {
      true
    }

    fn parse_is_and_where(&self) -> bool {
      true
    }

    fn is_and_where_error_recovery(&self) -> ParseErrorRecovery {
      ParseErrorRecovery::DiscardList
    }

    fn parse_part(&self) -> bool {
      true
    }

    fn parse_non_ts_pseudo_class(
      &self,
      location: SourceLocation,
      name: CowRcStr<'i>,
    ) -> Result<PseudoClass, SelectorParseError<'i>> {
      match_ignore_ascii_case! { &name,
          "hover" => return Ok(PseudoClass::Hover),
          "active" => return Ok(PseudoClass::Active),
          _ => {}
      }
      Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClass(name)))
    }

    fn parse_non_ts_functional_pseudo_class<'t>(
      &self,
      name: CowRcStr<'i>,
      parser: &mut CssParser<'i, 't>,
    ) -> Result<PseudoClass, SelectorParseError<'i>> {
      match_ignore_ascii_case! { &name,
          "lang" => {
              let lang = parser.expect_ident_or_string()?.as_ref().to_owned();
              return Ok(PseudoClass::Lang(lang));
          },
          _ => {}
      }
      Err(parser.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClass(name)))
    }

    fn parse_pseudo_element(
      &self,
      location: SourceLocation,
      name: CowRcStr<'i>,
    ) -> Result<PseudoElement, SelectorParseError<'i>> {
      match_ignore_ascii_case! { &name,
          "before" => return Ok(PseudoElement::Before),
          "after" => return Ok(PseudoElement::After),
          _ => {}
      }
      Err(location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoElement(name)))
    }

    fn default_namespace(&self) -> Option<DummyAtom> {
      self.default_ns.clone()
    }

    fn namespace_for_prefix(&self, prefix: &DummyAtom) -> Option<DummyAtom> {
      self.ns_prefixes.get(prefix).cloned()
    }
  }

  fn parse<'i>(input: &'i str) -> Result<SelectorList<'i, DummySelectorImpl>, SelectorParseError<'i>> {
    parse_ns(input, &DummyParser::default())
  }

  // fn parse_expected<'i, 'a>(
  //   input: &'i str,
  //   expected: Option<&'a str>,
  // ) -> Result<SelectorList<'i, DummySelectorImpl>, SelectorParseError<'i>> {
  //   parse_ns_expected(input, &DummyParser::default(), expected)
  // }

  fn parse_ns<'i>(
    input: &'i str,
    parser: &DummyParser,
  ) -> Result<SelectorList<'i, DummySelectorImpl>, SelectorParseError<'i>> {
    parse_ns_expected(input, parser, None)
  }

  fn parse_ns_expected<'i, 'a>(
    input: &'i str,
    parser: &DummyParser,
    expected: Option<&'a str>,
  ) -> Result<SelectorList<'i, DummySelectorImpl>, SelectorParseError<'i>> {
    let mut parser_input = ParserInput::new(input);
    let result = SelectorList::parse(
      parser,
      &mut CssParser::new(&mut parser_input),
      ParseErrorRecovery::DiscardList,
      NestingRequirement::None,
    );
    if let Ok(ref selectors) = result {
      assert_eq!(selectors.0.len(), 1);
      // We can't assume that the serialized parsed selector will equal
      // the input; for example, if there is no default namespace, '*|foo'
      // should serialize to 'foo'.
      assert_eq!(
        selectors.0[0].to_css_string(),
        match expected {
          Some(x) => x,
          None => input,
        }
      );
    }
    result
  }

  fn specificity(a: u32, b: u32, c: u32) -> u32 {
    a << 20 | b << 10 | c
  }

  #[test]
  fn test_empty() {
    let mut input = ParserInput::new(":empty");
    let list = SelectorList::parse(
      &DummyParser::default(),
      &mut CssParser::new(&mut input),
      ParseErrorRecovery::DiscardList,
      NestingRequirement::None,
    );
    assert!(list.is_ok());
  }

  const MATHML: &str = "http://www.w3.org/1998/Math/MathML";
  const SVG: &str = "http://www.w3.org/2000/svg";

  #[test]
  fn test_parsing() {
    assert!(parse("").is_err());
    assert!(parse(":lang(4)").is_err());
    assert!(parse(":lang(en US)").is_err());
    assert_eq!(
      parse("EeÉ"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::LocalName(LocalName {
          name: DummyAtom::from("EeÉ"),
          lower_name: DummyAtom::from("eeÉ"),
        })],
        specificity(0, 0, 1),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse("|e"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::ExplicitNoNamespace,
          Component::LocalName(LocalName {
            name: DummyAtom::from("e"),
            lower_name: DummyAtom::from("e"),
          }),
        ],
        specificity(0, 0, 1),
        Default::default(),
      )]))
    );
    // When the default namespace is not set, *| should be elided.
    // https://github.com/servo/servo/pull/17537
    // assert_eq!(
    //   parse_expected("*|e", Some("e")),
    //   Ok(SelectorList::from_vec(vec![Selector::from_vec(
    //     vec![Component::LocalName(LocalName {
    //       name: DummyAtom::from("e"),
    //       lower_name: DummyAtom::from("e"),
    //     })],
    //     specificity(0, 0, 1),
    //     Default::default(),
    //   )]))
    // );
    // When the default namespace is set, *| should _not_ be elided (as foo
    // is no longer equivalent to *|foo--the former is only for foo in the
    // default namespace).
    // https://github.com/servo/servo/issues/16020
    // assert_eq!(
    //   parse_ns(
    //     "*|e",
    //     &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
    //   ),
    //   Ok(SelectorList::from_vec(vec![Selector::from_vec(
    //     vec![
    //       Component::ExplicitAnyNamespace,
    //       Component::LocalName(LocalName {
    //         name: DummyAtom::from("e"),
    //         lower_name: DummyAtom::from("e"),
    //       }),
    //     ],
    //     specificity(0, 0, 1),
    //     Default::default(),
    //   )]))
    // );
    assert_eq!(
      parse("*"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::ExplicitUniversalType],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse("|*"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::ExplicitNoNamespace, Component::ExplicitUniversalType,],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    // assert_eq!(
    //   parse_expected("*|*", Some("*")),
    //   Ok(SelectorList::from_vec(vec![Selector::from_vec(
    //     vec![Component::ExplicitUniversalType],
    //     specificity(0, 0, 0),
    //     Default::default(),
    //   )]))
    // );
    assert_eq!(
      parse_ns(
        "*|*",
        &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org"))
      ),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::ExplicitAnyNamespace, Component::ExplicitUniversalType,],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse(".foo:lang(en-US)"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::Class(DummyAtom::from("foo")),
          Component::NonTSPseudoClass(PseudoClass::Lang("en-US".to_owned())),
        ],
        specificity(0, 2, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse("#bar"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::ID(DummyAtom::from("bar"))],
        specificity(1, 0, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse("e.foo#bar"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::LocalName(LocalName {
            name: DummyAtom::from("e"),
            lower_name: DummyAtom::from("e"),
          }),
          Component::Class(DummyAtom::from("foo")),
          Component::ID(DummyAtom::from("bar")),
        ],
        specificity(1, 1, 1),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse("e.foo #bar"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::LocalName(LocalName {
            name: DummyAtom::from("e"),
            lower_name: DummyAtom::from("e"),
          }),
          Component::Class(DummyAtom::from("foo")),
          Component::Combinator(Combinator::Descendant),
          Component::ID(DummyAtom::from("bar")),
        ],
        specificity(1, 1, 1),
        Default::default(),
      )]))
    );
    // Default namespace does not apply to attribute selectors
    // https://github.com/mozilla/servo/pull/1652
    let mut parser = DummyParser::default();
    assert_eq!(
      parse_ns("[Foo]", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::AttributeInNoNamespaceExists {
          local_name: DummyAtom::from("Foo"),
          local_name_lower: DummyAtom::from("foo"),
        }],
        specificity(0, 1, 0),
        Default::default(),
      )]))
    );
    assert!(parse_ns("svg|circle", &parser).is_err());
    parser.ns_prefixes.insert(DummyAtom("svg".into()), DummyAtom(SVG.into()));
    assert_eq!(
      parse_ns("svg|circle", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::Namespace(DummyAtom("svg".into()), SVG.into()),
          Component::LocalName(LocalName {
            name: DummyAtom::from("circle"),
            lower_name: DummyAtom::from("circle"),
          }),
        ],
        specificity(0, 0, 1),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse_ns("svg|*", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::Namespace(DummyAtom("svg".into()), SVG.into()),
          Component::ExplicitUniversalType,
        ],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    // Default namespace does not apply to attribute selectors
    // https://github.com/mozilla/servo/pull/1652
    // but it does apply to implicit type selectors
    // https://github.com/servo/rust-selectors/pull/82
    parser.default_ns = Some(MATHML.into());
    assert_eq!(
      parse_ns("[Foo]", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::DefaultNamespace(MATHML.into()),
          Component::AttributeInNoNamespaceExists {
            local_name: DummyAtom::from("Foo"),
            local_name_lower: DummyAtom::from("foo"),
          },
        ],
        specificity(0, 1, 0),
        Default::default(),
      )]))
    );
    // Default namespace does apply to type selectors
    assert_eq!(
      parse_ns("e", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::DefaultNamespace(MATHML.into()),
          Component::LocalName(LocalName {
            name: DummyAtom::from("e"),
            lower_name: DummyAtom::from("e"),
          }),
        ],
        specificity(0, 0, 1),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse_ns("*", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::DefaultNamespace(MATHML.into()),
          Component::ExplicitUniversalType,
        ],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse_ns("*|*", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::ExplicitAnyNamespace, Component::ExplicitUniversalType,],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    // Default namespace applies to universal and type selectors inside :not and :matches,
    // but not otherwise.
    assert_eq!(
      parse_ns(":not(.cl)", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::DefaultNamespace(MATHML.into()),
          Component::Negation(
            vec![Selector::from_vec(
              vec![Component::Class(DummyAtom::from("cl"))],
              specificity(0, 1, 0),
              Default::default(),
            )]
            .into_boxed_slice()
          ),
        ],
        specificity(0, 1, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse_ns(":not(*)", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::DefaultNamespace(MATHML.into()),
          Component::Negation(
            vec![Selector::from_vec(
              vec![
                Component::DefaultNamespace(MATHML.into()),
                Component::ExplicitUniversalType,
              ],
              specificity(0, 0, 0),
              Default::default(),
            )]
            .into_boxed_slice(),
          ),
        ],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse_ns(":not(e)", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::DefaultNamespace(MATHML.into()),
          Component::Negation(
            vec![Selector::from_vec(
              vec![
                Component::DefaultNamespace(MATHML.into()),
                Component::LocalName(LocalName {
                  name: DummyAtom::from("e"),
                  lower_name: DummyAtom::from("e"),
                }),
              ],
              specificity(0, 0, 1),
              Default::default(),
            ),]
            .into_boxed_slice()
          ),
        ],
        specificity(0, 0, 1),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse("[attr|=\"foo\"]"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::AttributeInNoNamespace {
          local_name: DummyAtom::from("attr"),
          operator: AttrSelectorOperator::DashMatch,
          value: DummyAttrValue::from("foo"),
          never_matches: false,
          case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
        }],
        specificity(0, 1, 0),
        Default::default(),
      )]))
    );
    // https://github.com/mozilla/servo/issues/1723
    assert_eq!(
      parse("::before"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::Combinator(Combinator::PseudoElement),
          Component::PseudoElement(PseudoElement::Before),
        ],
        specificity(0, 0, 1),
        SelectorFlags::HAS_PSEUDO,
      )]))
    );
    assert_eq!(
      parse("::before:hover"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::Combinator(Combinator::PseudoElement),
          Component::PseudoElement(PseudoElement::Before),
          Component::NonTSPseudoClass(PseudoClass::Hover),
        ],
        specificity(0, 1, 1),
        SelectorFlags::HAS_PSEUDO,
      )]))
    );
    assert_eq!(
      parse("::before:hover:hover"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::Combinator(Combinator::PseudoElement),
          Component::PseudoElement(PseudoElement::Before),
          Component::NonTSPseudoClass(PseudoClass::Hover),
          Component::NonTSPseudoClass(PseudoClass::Hover),
        ],
        specificity(0, 2, 1),
        SelectorFlags::HAS_PSEUDO,
      )]))
    );
    assert!(parse("::before:hover:lang(foo)").is_err());
    assert!(parse("::before:hover .foo").is_err());
    assert!(parse("::before .foo").is_err());
    assert!(parse("::before ~ bar").is_err());
    assert!(parse("::before:active").is_ok());

    // https://github.com/servo/servo/issues/15335
    assert!(parse(":: before").is_err());
    assert_eq!(
      parse("div ::after"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::LocalName(LocalName {
            name: DummyAtom::from("div"),
            lower_name: DummyAtom::from("div"),
          }),
          Component::Combinator(Combinator::Descendant),
          Component::Combinator(Combinator::PseudoElement),
          Component::PseudoElement(PseudoElement::After),
        ],
        specificity(0, 0, 2),
        SelectorFlags::HAS_PSEUDO,
      )]))
    );
    assert_eq!(
      parse("#d1 > .ok"),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![
          Component::ID(DummyAtom::from("d1")),
          Component::Combinator(Combinator::Child),
          Component::Class(DummyAtom::from("ok")),
        ],
        (1 << 20) + (1 << 10) + (0 << 0),
        Default::default(),
      )]))
    );
    parser.default_ns = None;
    assert!(parse(":not(#provel.old)").is_ok());
    assert!(parse(":not(#provel > old)").is_ok());
    assert!(parse("table[rules]:not([rules=\"none\"]):not([rules=\"\"])").is_ok());
    // https://github.com/servo/servo/issues/16017
    assert_eq!(
      parse_ns(":not(*)", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::Negation(
          vec![Selector::from_vec(
            vec![Component::ExplicitUniversalType],
            specificity(0, 0, 0),
            Default::default(),
          )]
          .into_boxed_slice()
        )],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    assert_eq!(
      parse_ns(":not(|*)", &parser),
      Ok(SelectorList::from_vec(vec![Selector::from_vec(
        vec![Component::Negation(
          vec![Selector::from_vec(
            vec![Component::ExplicitNoNamespace, Component::ExplicitUniversalType,],
            specificity(0, 0, 0),
            Default::default(),
          )]
          .into_boxed_slice(),
        )],
        specificity(0, 0, 0),
        Default::default(),
      )]))
    );
    // *| should be elided if there is no default namespace.
    // https://github.com/servo/servo/pull/17537
    // assert_eq!(
    //   parse_ns_expected(":not(*|*)", &parser, Some(":not(*)")),
    //   Ok(SelectorList::from_vec(vec![Selector::from_vec(
    //     vec![Component::Negation(
    //       vec![Selector::from_vec(
    //         vec![Component::ExplicitUniversalType],
    //         specificity(0, 0, 0),
    //         Default::default()
    //       )]
    //       .into_boxed_slice()
    //     )],
    //     specificity(0, 0, 0),
    //     Default::default(),
    //   )]))
    // );

    assert!(parse("::slotted()").is_err());
    assert!(parse("::slotted(div)").is_ok());
    assert!(parse("::slotted(div).foo").is_err());
    assert!(parse("::slotted(div + bar)").is_err());
    assert!(parse("::slotted(div) + foo").is_err());

    assert!(parse("::part()").is_err());
    assert!(parse("::part(42)").is_err());
    assert!(parse("::part(foo bar)").is_ok());
    assert!(parse("::part(foo):hover").is_ok());
    assert!(parse("::part(foo) + bar").is_err());

    assert!(parse("div ::slotted(div)").is_ok());
    assert!(parse("div + slot::slotted(div)").is_ok());
    assert!(parse("div + slot::slotted(div.foo)").is_ok());
    assert!(parse("slot::slotted(div,foo)::first-line").is_err());
    assert!(parse("::slotted(div)::before").is_ok());
    assert!(parse("slot::slotted(div,foo)").is_err());

    assert!(parse("foo:where()").is_err());
    assert!(parse("foo:where(div, foo, .bar baz)").is_ok());
    assert!(parse("foo:where(::before)").is_err());

    assert!(parse("foo::details-content").is_ok());
    assert!(parse("foo::target-text").is_ok());

    assert!(parse("select::picker").is_err());
    assert!(parse("::picker()").is_err());
    assert!(parse("::picker(select)").is_ok());
    assert!(parse("select::picker-icon").is_ok());
    assert!(parse("option::checkmark").is_ok());
  }

  #[test]
  fn test_pseudo_iter() {
    let selector = &parse("q::before").unwrap().0[0];
    assert!(!selector.is_universal());
    let mut iter = selector.iter();
    assert_eq!(iter.next(), Some(&Component::PseudoElement(PseudoElement::Before)));
    assert_eq!(iter.next(), None);
    let combinator = iter.next_sequence();
    assert_eq!(combinator, Some(Combinator::PseudoElement));
    assert!(matches!(iter.next(), Some(&Component::LocalName(..))));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_sequence(), None);
  }

  #[test]
  fn test_universal() {
    let selector = &parse_ns(
      "*|*::before",
      &DummyParser::default_with_namespace(DummyAtom::from("https://mozilla.org")),
    )
    .unwrap()
    .0[0];
    assert!(selector.is_universal());
  }

  #[test]
  fn test_empty_pseudo_iter() {
    let selector = &parse("::before").unwrap().0[0];
    assert!(selector.is_universal());
    let mut iter = selector.iter();
    assert_eq!(iter.next(), Some(&Component::PseudoElement(PseudoElement::Before)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_sequence(), Some(Combinator::PseudoElement));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_sequence(), None);
  }

  struct TestVisitor {
    seen: Vec<String>,
  }

  impl<'i> SelectorVisitor<'i> for TestVisitor {
    type Impl = DummySelectorImpl;

    fn visit_simple_selector(&mut self, s: &Component<DummySelectorImpl>) -> bool {
      let mut dest = String::new();
      s.to_css(&mut dest).unwrap();
      self.seen.push(dest);
      true
    }
  }

  #[test]
  fn visitor() {
    let mut test_visitor = TestVisitor { seen: vec![] };
    parse(":not(:hover) ~ label").unwrap().0[0].visit(&mut test_visitor);
    assert!(test_visitor.seen.contains(&":hover".into()));

    let mut test_visitor = TestVisitor { seen: vec![] };
    parse("::before:hover").unwrap().0[0].visit(&mut test_visitor);
    assert!(test_visitor.seen.contains(&":hover".into()));
  }
