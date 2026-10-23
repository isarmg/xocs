
  use crate::style_optimizer::values::color::RGBA;

  use super::*;

  fn test(source: &str, test: &str, expected: ParsedComponent) {
    let parsed = SyntaxString::parse_string(source).unwrap();

    let mut input = ParserInput::new(test);
    let mut parser = Parser::new(&mut input);
    let value = parsed.parse_value(&mut parser).unwrap();
    assert_eq!(value, expected);
  }

  fn parse_error_test(source: &str) {
    let res = SyntaxString::parse_string(source);
    match res {
      Ok(_) => unreachable!(),
      Err(_) => {}
    }
  }

  fn error_test(source: &str, test: &str) {
    let parsed = SyntaxString::parse_string(source).unwrap();
    let mut input = ParserInput::new(test);
    let mut parser = Parser::new(&mut input);
    let res = parsed.parse_value(&mut parser);
    match res {
      Ok(_) => unreachable!(),
      Err(_) => {}
    }
  }

  #[test]
  fn test_syntax() {
    test(
      "foo | <color>+ | <integer>",
      "foo",
      ParsedComponent::Literal("foo".into()),
    );

    test("foo|<color>+|<integer>", "foo", ParsedComponent::Literal("foo".into()));

    test("foo | <color>+ | <integer>", "2", ParsedComponent::Integer(2));

    test(
      "foo | <color>+ | <integer>",
      "red",
      ParsedComponent::Repeated {
        components: vec![ParsedComponent::Color(values::color::CssColor::RGBA(RGBA {
          red: 255,
          green: 0,
          blue: 0,
          alpha: 255,
        }))],
        multiplier: Multiplier::Space,
      },
    );

    test(
      "foo | <color>+ | <integer>",
      "red blue",
      ParsedComponent::Repeated {
        components: vec![
          ParsedComponent::Color(values::color::CssColor::RGBA(RGBA {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 255,
          })),
          ParsedComponent::Color(values::color::CssColor::RGBA(RGBA {
            red: 0,
            green: 0,
            blue: 255,
            alpha: 255,
          })),
        ],
        multiplier: Multiplier::Space,
      },
    );

    error_test("foo | <color>+ | <integer>", "2.5");

    error_test("foo | <color>+ | <integer>", "25px");

    error_test("foo | <color>+ | <integer>", "red, green");

    test(
      "foo | <color># | <integer>",
      "red, blue",
      ParsedComponent::Repeated {
        components: vec![
          ParsedComponent::Color(values::color::CssColor::RGBA(RGBA {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 255,
          })),
          ParsedComponent::Color(values::color::CssColor::RGBA(RGBA {
            red: 0,
            green: 0,
            blue: 255,
            alpha: 255,
          })),
        ],
        multiplier: Multiplier::Comma,
      },
    );

    error_test("foo | <color># | <integer>", "red green");

    test(
      "<length>",
      "25px",
      ParsedComponent::Length(values::length::Length::Value(values::length::LengthValue::Px(25.0))),
    );

    test(
      "<length>",
      "calc(25px + 25px)",
      ParsedComponent::Length(values::length::Length::Value(values::length::LengthValue::Px(50.0))),
    );

    test(
      "<length> | <percentage>",
      "25px",
      ParsedComponent::Length(values::length::Length::Value(values::length::LengthValue::Px(25.0))),
    );

    test(
      "<length> | <percentage>",
      "25%",
      ParsedComponent::Percentage(values::percentage::Percentage(0.25)),
    );

    error_test("<length> | <percentage>", "calc(100% - 25px)");

    test("foo | bar | baz", "bar", ParsedComponent::Literal("bar".into()));

    test(
      "<string>",
      "'foo'",
      ParsedComponent::String(values::string::CSSString("foo".into())),
    );

    test(
      "<custom-ident>",
      "hi",
      ParsedComponent::CustomIdent(values::ident::CustomIdent("hi".into())),
    );

    parse_error_test("<transform-list>#");
    parse_error_test("<color");
    parse_error_test("color>");
  }
