
    use super::*;

    #[track_caller]
    fn check_complex(s: &str, expected: &[&str], segmenter: ComplexPayloadsBorrowed<'_>) {
        use itertools::Itertools;

        let segments = [0]
            .into_iter()
            .chain(segmenter.segment_str(s))
            .tuple_windows()
            .map(|(a, b)| &s[a..b])
            .collect::<Vec<_>>();
        assert_eq!(segments, expected, "{s}");

        // let segments = segmenter
        //     .segment_utf8(s.as_bytes())
        //     .tuple_windows()
        //     .map(|(a, b)| &s[a..b])
        //     .collect::<Vec<_>>();
        // assert_eq!(segments, expected, "{s}");

        let utf16: Vec<u16> = s.encode_utf16().collect();
        let expected = expected
            .iter()
            .copied()
            .map(|s| s.encode_utf16().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let iter = [0]
            .into_iter()
            .chain(segmenter.segment_utf16(&utf16))
            .tuple_windows()
            .map(|(a, b)| &utf16[a..b])
            .collect::<Vec<_>>();
        assert_eq!(iter, expected, "{s}");
    }

    #[test]
    fn thai() {
        let mut lstm = ComplexPayloadsBorrowed::new();
        lstm.with_southeast_asian_lstms();
        let mut dict = ComplexPayloadsBorrowed::new();
        dict.with_southeast_asian_dictionaries();

        check_complex("ภาษาไทยภาษาไทย", &["ภาษา", "ไทย", "ภาษา", "ไทย"], lstm);
        check_complex("ภาษาไทยภาษาไทย", &["ภาษา", "ไทย", "ภาษา", "ไทย"], dict);
    }

    #[test]
    fn mixed() {
        let mut lstm = ComplexPayloadsBorrowed::new();
        lstm.with_southeast_asian_lstms();
        lstm.with_japanese_dictionary();

        let mut dict = ComplexPayloadsBorrowed::new();
        dict.with_southeast_asian_dictionaries();
        dict.with_japanese_dictionary();

        check_complex("ภาษาไทย龟山岛", &["ภาษา", "ไทย", "龟山岛"], lstm);
        check_complex("ภาษาไทย龟山岛", &["ภาษา", "ไทย", "龟山岛"], dict);

        check_complex(
            "こんにちは世界ภาษาไทย",
            &["こんにちは", "世界", "ภาษา", "ไทย"],
            lstm,
        );
        check_complex(
            "こんにちは世界ภาษาไทย",
            &["こんにちは", "世界", "ภาษา", "ไทย"],
            dict,
        );
    }
