
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn all_boundaries_in_iter() {
        let all = Boundary::all();
        for boundary in Boundary::iter() {
            assert!(all.contains(&boundary));
        }
    }

    #[test]
    fn split_on_delims() {
        assert_eq!(
            vec!["my", "word", "list", "separated", "by", "delims"],
            split("my_word-list separated-by_delims", &Boundary::delims())
        )
    }

    #[test]
    fn boundaries_found_in_string() {
        use Boundary::*;
        assert_eq!(
            vec![UpperLower],
            Boundary::list_from(".Aaaa")
        );
        assert_eq!(
            vec![LowerUpper, UpperLower, LowerDigit],
            Boundary::list_from("a8.Aa.aA")
        );
        assert_eq!(
            Boundary::digits(),
            Boundary::list_from("b1B1b")
        );
        assert_eq!(
            vec![Hyphen, Underscore, Space, Acronym],
            Boundary::list_from("AAa -_")
        );
    }
