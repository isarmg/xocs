
  use super::*;
  use crate::style_optimizer::{
    stylesheet::PrinterOptions,
    targets::{Browsers, Targets},
  };

  fn parse(s: &str) -> MediaQuery<'_> {
    let mut input = ParserInput::new(&s);
    let mut parser = Parser::new(&mut input);
    MediaQuery::parse_with_options(&mut parser, &ParserOptions::default()).unwrap()
  }

  fn and(a: &str, b: &str) -> String {
    let mut a = parse(a);
    let b = parse(b);
    a.and(&b).unwrap();
    a.to_css_string(PrinterOptions::default()).unwrap()
  }

  #[test]
  fn test_and() {
    assert_eq!(and("(min-width: 250px)", "(color)"), "(width >= 250px) and (color)");
    assert_eq!(
      and("(min-width: 250px) or (color)", "(orientation: landscape)"),
      "((width >= 250px) or (color)) and (orientation: landscape)"
    );
    assert_eq!(
      and("(min-width: 250px) and (color)", "(orientation: landscape)"),
      "(width >= 250px) and (color) and (orientation: landscape)"
    );
    assert_eq!(and("all", "print"), "print");
    assert_eq!(and("print", "all"), "print");
    assert_eq!(and("all", "not print"), "not print");
    assert_eq!(and("not print", "all"), "not print");
    assert_eq!(and("not all", "print"), "not all");
    assert_eq!(and("print", "not all"), "not all");
    assert_eq!(and("print", "screen"), "not all");
    assert_eq!(and("not print", "screen"), "screen");
    assert_eq!(and("print", "not screen"), "print");
    assert_eq!(and("not screen", "print"), "print");
    assert_eq!(and("not screen", "not all"), "not all");
    assert_eq!(and("print", "(min-width: 250px)"), "print and (width >= 250px)");
    assert_eq!(and("(min-width: 250px)", "print"), "print and (width >= 250px)");
    assert_eq!(
      and("print and (min-width: 250px)", "(color)"),
      "print and (width >= 250px) and (color)"
    );
    assert_eq!(and("all", "only screen"), "only screen");
    assert_eq!(and("only screen", "all"), "only screen");
    assert_eq!(and("print", "print"), "print");
  }

  #[test]
  fn test_negated_interval_parens() {
    let media_query = parse("screen and not (200px <= width < 500px)");
    let printer_options = PrinterOptions {
      targets: Targets {
        browsers: Some(Browsers {
          chrome: Some(95 << 16),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    };
    assert_eq!(
      media_query.to_css_string(printer_options).unwrap(),
      "screen and not ((min-width: 200px) and (not (min-width: 500px)))"
    );
  }
