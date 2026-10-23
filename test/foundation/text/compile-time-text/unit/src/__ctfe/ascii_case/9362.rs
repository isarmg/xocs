#[test]
fn test_conv_ascii_case() {
    macro_rules! test_conv_ascii_case {
        ($v: tt, $a: expr, $b: expr $(,)?) => {{
            const A: &str = $a;
            const B: &str = convert_ascii_case!($v, A);
            assert_eq!(B, $b);
            test_conv_ascii_case!(heck, $v, $a, $b);
        }};
        (heck, camel, $a: expr, $b: expr) => {{
            use heck::CamelCase;
            let c: String = $a.to_camel_case();
            assert_eq!(c.as_str(), $b, "heck");
        }};
        (heck, snake, $a: expr, $b: expr) => {{
            use heck::SnakeCase;
            let c: String = $a.to_snake_case();
            assert_eq!(c.as_str(), $b, "heck");
        }};
        (heck, kebab, $a: expr, $b: expr) => {{
            use heck::KebabCase;
            let c: String = $a.to_kebab_case();
            assert_eq!(c.as_str(), $b, "heck");
        }};
        (heck, shouty_snake, $a: expr, $b: expr) => {{
            use heck::ShoutySnakeCase;
            let c: String = $a.to_shouty_snake_case();
            assert_eq!(c.as_str(), $b, "heck");
        }};
        (heck, shouty_kebab, $a: expr, $b: expr) => {{
            use heck::ShoutyKebabCase;
            let c: String = $a.to_shouty_kebab_case();
            assert_eq!(c.as_str(), $b, "heck");
        }};
    }

    {
        const S: &str = "b.8";
        test_conv_ascii_case!(camel, S, "B8");
        test_conv_ascii_case!(snake, S, "b_8");
        test_conv_ascii_case!(kebab, S, "b-8");
        test_conv_ascii_case!(shouty_snake, S, "B_8");
        test_conv_ascii_case!(shouty_kebab, S, "B-8");
    }

    {
        const S: &str = "Hello World123!XMLHttp我4t5.c6.7b.8";
        test_conv_ascii_case!(camel, S, "HelloWorld123XmlHttp我4t5C6.7b8");
        test_conv_ascii_case!(snake, S, "hello_world123_xml_http_我_4t5_c6.7b_8");
        test_conv_ascii_case!(kebab, S, "hello-world123-xml-http-我-4t5-c6.7b-8");
        test_conv_ascii_case!(shouty_snake, S, "HELLO_WORLD123_XML_HTTP_我_4T5_C6.7B_8");
        test_conv_ascii_case!(shouty_kebab, S, "HELLO-WORLD123-XML-HTTP-我-4T5-C6.7B-8");
    }
    {
        const S: &str = "XMLHttpRequest";
        test_conv_ascii_case!(camel, S, "XmlHttpRequest");
        test_conv_ascii_case!(snake, S, "xml_http_request");
        test_conv_ascii_case!(kebab, S, "xml-http-request");
        test_conv_ascii_case!(shouty_snake, S, "XML_HTTP_REQUEST");
        test_conv_ascii_case!(shouty_kebab, S, "XML-HTTP-REQUEST");
    }
    {
        const S: &str = "  hello world  ";
        test_conv_ascii_case!(camel, S, "HelloWorld");
        test_conv_ascii_case!(snake, S, "hello_world");
        test_conv_ascii_case!(kebab, S, "hello-world");
        test_conv_ascii_case!(shouty_snake, S, "HELLO_WORLD");
        test_conv_ascii_case!(shouty_kebab, S, "HELLO-WORLD");
    }
    {
        const S: &str = "";
        test_conv_ascii_case!(camel, S, "");
        test_conv_ascii_case!(snake, S, "");
        test_conv_ascii_case!(kebab, S, "");
        test_conv_ascii_case!(shouty_snake, S, "");
        test_conv_ascii_case!(shouty_kebab, S, "");
    }
    {
        const S: &str = "_";
        test_conv_ascii_case!(camel, S, "");
        test_conv_ascii_case!(snake, S, "");
        test_conv_ascii_case!(kebab, S, "");
        test_conv_ascii_case!(shouty_snake, S, "");
        test_conv_ascii_case!(shouty_kebab, S, "");
    }
    {
        const S: &str = "1.2E3";
        test_conv_ascii_case!(camel, S, "1.2e3");
        test_conv_ascii_case!(snake, S, "1.2e3");
        test_conv_ascii_case!(kebab, S, "1.2e3");
        test_conv_ascii_case!(shouty_snake, S, "1.2E3");
        test_conv_ascii_case!(shouty_kebab, S, "1.2E3");
    }
    {
        const S: &str = "__a__b-c__d__";
        test_conv_ascii_case!(camel, S, "ABCD");
        test_conv_ascii_case!(snake, S, "a_b_c_d");
        test_conv_ascii_case!(kebab, S, "a-b-c-d");
        test_conv_ascii_case!(shouty_snake, S, "A_B_C_D");
        test_conv_ascii_case!(shouty_kebab, S, "A-B-C-D");
    }
    {
        const S: &str = "futures-core123";
        test_conv_ascii_case!(camel, S, "FuturesCore123");
        test_conv_ascii_case!(snake, S, "futures_core123");
        test_conv_ascii_case!(kebab, S, "futures-core123");
        test_conv_ascii_case!(shouty_snake, S, "FUTURES_CORE123");
        test_conv_ascii_case!(shouty_kebab, S, "FUTURES-CORE123");
    }
}
