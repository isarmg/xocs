#[cfg(test)]
    #[test]
    fn test_read_word() {
        assert_eq!(
            read_word("       option timezone 'CST-8'\n").unwrap(),
            Some(("option", "timezone 'CST-8'\n")),
        );
        assert_eq!(
            read_word("timezone 'CST-8'\n").unwrap(),
            Some(("timezone", "'CST-8'\n")),
        );
        assert_eq!(read_word("'CST-8'\n").unwrap(), Some(("CST-8", "\n")));
        assert_eq!(read_word("\n").unwrap(), None);

        assert_eq!(
            read_word(r#""time 'Zone'""#).unwrap(),
            Some(("time 'Zone'", "")),
        );

        assert_eq!(read_word("'CST-8").unwrap_err(), BrokenQuote);
    }
