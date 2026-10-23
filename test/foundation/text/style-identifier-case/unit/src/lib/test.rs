
    use super::*;
    use strum::IntoEnumIterator;

    fn possible_cases(s: &str) -> Vec<Case> {
        Case::deterministic_cases()
            .into_iter()
            .filter(|case| s.from_case(*case).to_case(*case) == s)
            .collect()
    }

    #[test]
    fn lossless_against_lossless() {
        let examples = vec![
            (Case::Lower, "my variable 22 name"),
            (Case::Upper, "MY VARIABLE 22 NAME"),
            (Case::Title, "My Variable 22 Name"),
            (Case::Camel, "myVariable22Name"),
            (Case::Pascal, "MyVariable22Name"),
            (Case::Snake, "my_variable_22_name"),
            (Case::UpperSnake, "MY_VARIABLE_22_NAME"),
            (Case::Kebab, "my-variable-22-name"),
            (Case::Cobol, "MY-VARIABLE-22-NAME"),
            (Case::Toggle, "mY vARIABLE 22 nAME"),
            (Case::Train, "My-Variable-22-Name"),
            (Case::Alternating, "mY vArIaBlE 22 nAmE"),
        ];

        for (case_a, str_a) in examples.iter() {
            for (case_b, str_b) in examples.iter() {
                assert_eq!(*str_a, str_b.from_case(*case_b).to_case(*case_a))
            }
        }
    }

    #[test]
    fn obvious_default_parsing() {
        let examples = vec![
            "SuperMario64Game",
            "super-mario64-game",
            "superMario64 game",
            "Super Mario 64_game",
            "SUPERMario 64-game",
            "super_mario-64 game",
        ];

        for example in examples {
            assert_eq!("super_mario_64_game", example.to_case(Case::Snake));
        }
    }

    #[test]
    fn multiline_strings() {
        assert_eq!("One\ntwo\nthree", "one\ntwo\nthree".to_case(Case::Title));
    }

    #[test]
    fn camel_case_acroynms() {
        assert_eq!(
            "xml_http_request",
            "XMLHttpRequest".from_case(Case::Camel).to_case(Case::Snake)
        );
        assert_eq!(
            "xml_http_request",
            "XMLHttpRequest"
                .from_case(Case::UpperCamel)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "xml_http_request",
            "XMLHttpRequest"
                .from_case(Case::Pascal)
                .to_case(Case::Snake)
        );
    }

    #[test]
    fn leading_tailing_delimeters() {
        assert_eq!(
            "leading_underscore",
            "_leading_underscore"
                .from_case(Case::Snake)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "tailing_underscore",
            "tailing_underscore_"
                .from_case(Case::Snake)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "leading_hyphen",
            "-leading-hyphen"
                .from_case(Case::Kebab)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "tailing_hyphen",
            "tailing-hyphen-"
                .from_case(Case::Kebab)
                .to_case(Case::Snake)
        );
    }

    #[test]
    fn double_delimeters() {
        assert_eq!(
            "many_underscores",
            "many___underscores"
                .from_case(Case::Snake)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "many-underscores",
            "many---underscores"
                .from_case(Case::Kebab)
                .to_case(Case::Kebab)
        );
    }

    #[test]
    fn early_word_boundaries() {
        assert_eq!(
            "a_bagel",
            "aBagel".from_case(Case::Camel).to_case(Case::Snake)
        );
    }

    #[test]
    fn late_word_boundaries() {
        assert_eq!(
            "team_a",
            "teamA".from_case(Case::Camel).to_case(Case::Snake)
        );
    }

    #[test]
    fn empty_string() {
        for (case_a, case_b) in Case::iter().zip(Case::iter()) {
            assert_eq!("", "".from_case(case_a).to_case(case_b));
        }
    }

    #[test]
    fn owned_string() {
        assert_eq!(
            "test_variable",
            String::from("TestVariable").to_case(Case::Snake)
        )
    }

    #[test]
    fn default_all_boundaries() {
        assert_eq!(
            "abc_abc_abc_abc_abc_abc",
            "ABC-abc_abcAbc ABCAbc".to_case(Case::Snake)
        );
    }

    #[test]
    fn alternating_ignore_symbols() {
        assert_eq!("tHaT's", "that's".to_case(Case::Alternating));
    }

    #[test]
    fn string_is_snake() {
        assert!("im_snake_case".is_case(Case::Snake));
        assert!(!"im_NOTsnake_case".is_case(Case::Snake));
    }

    #[test]
    fn string_is_kebab() {
        assert!("im-kebab-case".is_case(Case::Kebab));
        assert!(!"im_not_kebab".is_case(Case::Kebab));
    }

    #[test]
    fn remove_boundaries() {
        assert_eq!(
            "m02_s05_binary_trees.pdf",
            "M02S05BinaryTrees.pdf"
                .from_case(Case::Pascal)
                .without_boundaries(&[Boundary::UpperDigit])
                .to_case(Case::Snake)
        );
    }

    #[test]
    fn with_boundaries() {
        assert_eq!(
            "my-dumb-file-name",
            "my_dumbFileName"
                .with_boundaries(&[Boundary::Underscore, Boundary::LowerUpper])
                .to_case(Case::Kebab)
        );
    }

    #[cfg(feature = "random")]
    #[test]
    fn random_case_boundaries() {
        for random_case in Case::random_cases() {
            assert_eq!(
                "split_by_spaces",
                "Split By Spaces"
                    .from_case(random_case)
                    .to_case(Case::Snake)
            );
        }
    }

    #[test]
    fn multiple_from_case() {
        assert_eq!(
            "longtime_nosee",
            "LongTime NoSee"
                .from_case(Case::Camel)
                .from_case(Case::Title)
                .to_case(Case::Snake),
        )
    }

    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn detect_many_cases() {
        let lower_cases_vec = possible_cases(&"asef");
        let lower_cases_set = HashSet::from_iter(lower_cases_vec.into_iter());
        let mut actual = HashSet::new();
        actual.insert(Case::Lower);
        actual.insert(Case::Camel);
        actual.insert(Case::Snake);
        actual.insert(Case::Kebab);
        actual.insert(Case::Flat);
        assert_eq!(lower_cases_set, actual);

        let lower_cases_vec = possible_cases(&"asefCase");
        let lower_cases_set = HashSet::from_iter(lower_cases_vec.into_iter());
        let mut actual = HashSet::new();
        actual.insert(Case::Camel);
        assert_eq!(lower_cases_set, actual);
    }

    #[test]
    fn detect_each_case() {
        let s = "My String Identifier".to_string();
        for case in Case::deterministic_cases() {
            let new_s = s.from_case(case).to_case(case);
            let possible = possible_cases(&new_s);
            println!("{} {:?} {:?}", new_s, case, possible);
            assert!(possible.iter().any(|c| c == &case));
        }
    }

    // From issue https://github.com/rutrum/convert-case/issues/8
    #[test]
    fn accent_mark() {
        let s = "música moderna".to_string();
        assert_eq!("MúsicaModerna", s.to_case(Case::Pascal));
    }

    // From issue https://github.com/rutrum/convert-case/issues/4
    #[test]
    fn russian() {
        let s = "ПЕРСПЕКТИВА24".to_string();
        let _n = s.to_case(Case::Title);
    }
