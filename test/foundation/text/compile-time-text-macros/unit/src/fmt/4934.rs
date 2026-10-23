#[test]
fn test_parse_fmt() {
    {
        let s = "";
        assert!(parse_fmt_string(s).unwrap().is_empty());
    }

    {
        let s = "{}";
        let parts = parse_fmt_string(s).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(matches!(parts[0].pos, Some(0)))
    }

    {
        let s = "{1} {} {0} {}";
        let parts = parse_fmt_string(s).unwrap();
        assert_eq!(parts.len(), 7);
        assert!(matches!(parts[0].pos, Some(1)));
        assert!(matches!(parts[1].literal_str().unwrap(), " "));
        assert!(matches!(parts[2].pos, Some(0)));
        assert!(matches!(parts[3].literal_str().unwrap(), " "));
        assert!(matches!(parts[4].pos, Some(0)));
        assert!(matches!(parts[5].literal_str().unwrap(), " "));
        assert!(matches!(parts[6].pos, Some(1)));
    }

    {
        let s = "{argument}";
        let parts = parse_fmt_string(s).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].named_ident().unwrap(), "argument");
    }

    {
        let s = "{name} {}";
        let parts = parse_fmt_string(s).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].named_ident().unwrap(), "name");
        assert!(matches!(parts[1].literal_str().unwrap(), " "));
        assert!(matches!(parts[2].pos, Some(0)));
    }

    {
        let s = "{a} {c} {b}";
        let parts = parse_fmt_string(s).unwrap();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].named_ident().unwrap(), "a");
        assert!(matches!(parts[1].literal_str().unwrap(), " "));
        assert_eq!(parts[2].named_ident().unwrap(), "c");
        assert!(matches!(parts[3].literal_str().unwrap(), " "));
        assert_eq!(parts[4].named_ident().unwrap(), "b");
    }

    {
        let s = "{{}}";
        let parts = parse_fmt_string(s).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].literal_str().unwrap(), "{}");
    }
}
