
    use super::*;

    #[test]
    fn parse_version() {
        assert_eq!(Ok(Version(1, 0, 0)), "1".parse());
        assert_eq!(Ok(Version(1, 2, 0)), "1.2".parse());
        assert_eq!(Ok(Version(1, 2, 3)), "1.2.3".parse());
        assert_eq!(Ok(Version(12, 34, 56)), "12.34.56".parse());

        assert_eq!(Ok(Version(1, 0, 0)), "1-2".parse());
        assert_eq!(Ok(Version(1, 2, 0)), "1.2-1.3".parse());
        assert_eq!(Ok(Version(1, 2, 3)), "1.2.3-1.2.4".parse());
        assert_eq!(Ok(Version(12, 34, 56)), "12.34.56-78.9".parse());

        assert!("tp".parse::<Version>().is_err());
    }
