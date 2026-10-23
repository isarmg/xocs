
    use super::*;
    use crate::LineSegmenterBorrowed;
    use crate::*;

    include!("../../../integration/helpers.rs.raw");

    #[test]
    fn test_mandatory() {
        let mut actual_breaks = LineSegmenter::new_neo_for_non_complex_scripts(Default::default())
            .segment_str("this has a mandatory\nline break");

        assert_eq!(actual_breaks.next(), Some(0));
        assert_eq!(actual_breaks.is_mandatory(), false);
        assert_eq!(actual_breaks.next(), Some(5));
        assert_eq!(actual_breaks.is_mandatory(), false);
        assert_eq!(actual_breaks.next(), Some(9));
        assert_eq!(actual_breaks.is_mandatory(), false);
        assert_eq!(actual_breaks.next(), Some(11));
        assert_eq!(actual_breaks.is_mandatory(), false);
        assert_eq!(actual_breaks.next(), Some(21));
        assert_eq!(actual_breaks.is_mandatory(), true);
        assert_eq!(actual_breaks.next(), Some(26));
        assert_eq!(actual_breaks.is_mandatory(), false);
        assert_eq!(actual_breaks.next(), Some(31));
        assert_eq!(actual_breaks.is_mandatory(), true);
        assert_eq!(actual_breaks.next(), None);
    }

    #[test]
    fn linebreak() {
        let segmenter = LineSegmenter::new_dictionary(Default::default());

        check_line("hello world", &["hello ", "world"], segmenter);

        check_line("$10 $10", &["$10 ", "$10"], segmenter);

        // LB10

        // LB14
        check_line("[  abc def", &["[  abc ", "def"], segmenter);

        // LB15 used to prevent the break at 6, but has been removed in Unicode 15.1.
        check_line("abc\u{0022}  (def", &["abc\u{0022}  ", "(def"], segmenter);

        // Instead, in Unicode 15.1, LB15a and LB15b prevent these breaks.
        check_line("« miaou »", &["« miaou »"], segmenter);

        // But not these:
        check_line(
            "Die Katze hat »miau« gesagt.",
            &["Die ", "Katze ", "hat ", "»miau« ", "gesagt."],
            segmenter,
        );

        // LB16
        check_line("\u{0029}\u{203C}", &["\u{0029}\u{203C}"], segmenter);
        check_line("\u{0029}  \u{203C}", &["\u{0029}  \u{203C}"], segmenter);

        // LB17
        check_line("\u{2014}\u{2014}aa", &["\u{2014}\u{2014}", "aa"], segmenter);
        check_line(
            "\u{2014}  \u{2014}aa",
            &["\u{2014}  \u{2014}", "aa"],
            segmenter,
        );

        check_line(
            "\u{2014}\u{2014}  \u{2014}\u{2014}123 abc",
            &["\u{2014}\u{2014}  \u{2014}\u{2014}", "123 ", "abc"],
            segmenter,
        );

        // LB25
        check_line("(0,1)+(2,3)", &["(0,1)+(2,3)"], segmenter);

        check_line("——  ——123 abc", &["——  ——", "123 ", "abc"], segmenter);
        check_line(
            "\u{1F3FB} \u{1F3FB}",
            &["\u{1F3FB} ", "\u{1F3FB}"],
            segmenter,
        );
    }

    #[test]
    fn linebreak_17() {
        let segmenter = LineSegmenter::new_17_for_non_complex_scripts(Default::default());

        check_line("hello world", &["hello ", "world"], segmenter);

        check_line("$10 $10", &["$10 ", "$10"], segmenter);

        // LB10

        // LB14
        check_line("[  abc def", &["[  abc ", "def"], segmenter);

        // LB15 used to prevent the break at 6, but has been removed in Unicode 15.1.
        check_line("abc\u{0022}  (def", &["abc\u{0022}  ", "(def"], segmenter);

        // Instead, in Unicode 15.1, LB15a and LB15b prevent these breaks.
        check_line("« miaou »", &["« miaou »"], segmenter);

        // But not these:
        check_line(
            "Die Katze hat »miau« gesagt.",
            &["Die ", "Katze ", "hat ", "»miau« ", "gesagt."],
            segmenter,
        );

        // LB16
        check_line("\u{0029}\u{203C}", &["\u{0029}\u{203C}"], segmenter);
        check_line("\u{0029}  \u{203C}", &["\u{0029}  \u{203C}"], segmenter);

        // LB17
        check_line("\u{2014}\u{2014}aa", &["\u{2014}\u{2014}", "aa"], segmenter);
        check_line(
            "\u{2014}  \u{2014}aa",
            &["\u{2014}  \u{2014}", "aa"],
            segmenter,
        );

        check_line(
            "\u{2014}\u{2014}  \u{2014}\u{2014}123 abc",
            &["\u{2014}\u{2014}  \u{2014}\u{2014}", "123 ", "abc"],
            segmenter,
        );

        // LB25
        check_line("(0,1)+(2,3)", &["(0,1)+(2,3)"], segmenter);

        check_line("——  ——123 abc", &["——  ——", "123 ", "abc"], segmenter);
        check_line(
            "\u{1F3FB} \u{1F3FB}",
            &["\u{1F3FB} ", "\u{1F3FB}"],
            segmenter,
        );
    }

    #[test]
    fn linebreak_neo() {
        let segmenter = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());

        check_line("hello world", &["hello ", "world"], segmenter);

        check_line("$10 $10", &["$10 ", "$10"], segmenter);

        // LB10

        // LB14
        check_line("[  abc def", &["[  abc ", "def"], segmenter);

        // LB15 used to prevent the break at 6, but has been removed in Unicode 15.1.
        check_line("abc\u{0022}  (def", &["abc\u{0022}  ", "(def"], segmenter);

        // Instead, in Unicode 15.1, LB15a and LB15b prevent these breaks.
        check_line("« miaou »", &["« miaou »"], segmenter);

        // But not these:
        check_line(
            "Die Katze hat »miau« gesagt.",
            &["Die ", "Katze ", "hat ", "»miau« ", "gesagt."],
            segmenter,
        );

        // LB16
        check_line("\u{0029}\u{203C}", &["\u{0029}\u{203C}"], segmenter);
        check_line("\u{0029}  \u{203C}", &["\u{0029}  \u{203C}"], segmenter);

        // LB17
        check_line("\u{2014}\u{2014}aa", &["\u{2014}\u{2014}", "aa"], segmenter);
        check_line(
            "\u{2014}  \u{2014}aa",
            &["\u{2014}  \u{2014}", "aa"],
            segmenter,
        );

        check_line(
            "\u{2014}\u{2014}  \u{2014}\u{2014}123 abc",
            &["\u{2014}\u{2014}  \u{2014}\u{2014}", "123 ", "abc"],
            segmenter,
        );

        // LB25
        check_line("(0,1)+(2,3)", &["(0,1)+(2,3)"], segmenter);

        check_line("——  ——123 abc", &["——  ——", "123 ", "abc"], segmenter);
        check_line(
            "\u{1F3FB} \u{1F3FB}",
            &["\u{1F3FB} ", "\u{1F3FB}"],
            segmenter,
        );
    }

    #[test]
    fn thai_line_break() {
        check_line(
            "ภาษาไทยภาษาไทย",
            &["ภาษา", "ไทย", "ภาษา", "ไทย"],
            LineSegmenter::new_lstm(Default::default()),
        );

        check_line(
            "ภาษาไทยภาษาไทย",
            &["ภาษา", "ไทย", "ภาษา", "ไทย"],
            LineSegmenter::new_dictionary(Default::default()),
        );

        check_line(
            "ภาษา",
            &["ภาษา"],
            LineSegmenter::new_lstm(Default::default()),
        );

        check_line(
            "ภาษา",
            &["ภาษา"],
            LineSegmenter::new_dictionary(Default::default()),
        );
    }

    #[test]
    fn thai_line_break_17() {
        check_line(
            "ภาษาไทยภาษาไทย",
            &["ภาษา", "ไทย", "ภาษา", "ไทย"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "ภาษาไทยภาษาไทย",
            &["ภาษา", "ไทย", "ภาษา", "ไทย"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );

        check_line("ภาษา", &["ภาษา"], {
            let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
            s.load_lstm();
            s
        });

        check_line("ภาษา", &["ภาษา"], {
            let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
            s.load_dictionary();
            s
        });
    }

    #[test]
    fn thai_line_break_neo() {
        check_line(
            "ภาษาไทยภาษาไทย",
            &["ภาษา", "ไทย", "ภาษา", "ไทย"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "ภาษาไทยภาษาไทย",
            &["ภาษา", "ไทย", "ภาษา", "ไทย"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );

        check_line("ภาษา", &["ภาษา"], {
            let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
            s.load_lstm();
            s
        });

        check_line("ภาษา", &["ภาษา"], {
            let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
            s.load_dictionary();
            s
        });

        // # 8243
        check_line(
            "ก\u{2060}รุ\u{2060}ง",
            &["ก\u{2060}รุ\u{2060}ง"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn burmese_line_break() {
        // "Burmese Language" in Burmese

        check_line(
            "မြန်မာဘာသာစကား",
            &["မြန်", "မာ", "ဘာသာ", "စကား"],
            LineSegmenter::new_lstm(Default::default()),
        );

        check_line(
            "မြန်မာဘာသာစကား",
            &["မြန်မာဘာသာ", "စကား"],
            LineSegmenter::new_dictionary(Default::default()),
        );
    }

    #[test]
    fn burmese_line_break_17() {
        // "Burmese Language" in Burmese

        check_line(
            "မြန်မာဘာသာစကား",
            &["မြန်", "မာ", "ဘာသာ", "စကား"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "မြန်မာဘာသာစကား",
            &["မြန်မာဘာသာ", "စကား"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn burmese_line_break_neo() {
        // "Burmese Language" in Burmese

        check_line(
            "မြန်မာဘာသာစကား",
            &["မြန်", "မာ", "ဘာသာ", "စကား"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "မြန်မာဘာသာစကား",
            &["မြန်မာဘာသာ", "စကား"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn khmer_line_break() {
        check_line(
            "សេចក្ដីប្រកាសជាសកលស្ដីពីសិទ្ធិមនុស្ស",
            &["សេចក្ដីប្រកាស", "ជាស", "កល", "ស្ដីពី", "សិទ្ធិមនុស្ស"],
            LineSegmenter::new_lstm(Default::default()),
        );

        check_line(
            "សេចក្ដីប្រកាសជាសកលស្ដីពីសិទ្ធិមនុស្ស",
            &["សេចក្ដីប្រកាស", "ជាស", "កល", "ស្ដីពី", "សិទ្ធិមនុស្ស"],
            LineSegmenter::new_dictionary(Default::default()),
        );
    }

    #[test]
    fn khmer_line_break_17() {
        check_line(
            "សេចក្ដីប្រកាសជាសកលស្ដីពីសិទ្ធិមនុស្ស",
            &["សេចក្ដីប្រកាស", "ជាស", "កល", "ស្ដីពី", "សិទ្ធិមនុស្ស"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "សេចក្ដីប្រកាសជាសកលស្ដីពីសិទ្ធិមនុស្ស",
            &["សេចក្ដីប្រកាស", "ជាស", "កល", "ស្ដីពី", "សិទ្ធិមនុស្ស"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn khmer_line_break_neo() {
        check_line(
            "សេចក្ដីប្រកាសជាសកលស្ដីពីសិទ្ធិមនុស្ស",
            &["សេចក្ដីប្រកាស", "ជាស", "កល", "ស្ដីពី", "សិទ្ធិមនុស្ស"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "សេចក្ដីប្រកាសជាសកលស្ដីពីសិទ្ធិមនុស្ស",
            &["សេចក្ដីប្រកាស", "ជាស", "កល", "ស្ដីពី", "សិទ្ធិមនុស្ស"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );

        // #7218
        check_line(
            "អស់ នឹង មាន",
            &["អស់ ", "នឹង ", "មាន"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn lao_line_break() {
        check_line(
            "ກ່ຽວກັບສິດຂອງມະນຸດ",
            &["ກ່ຽວ", "ກັບ", "ສິດ", "ຂອງ", "ມະນຸດ"],
            LineSegmenter::new_lstm(Default::default()),
        );

        check_line(
            "ກ່ຽວກັບສິດຂອງມະນຸດ",
            &["ກ່ຽວກັບ", "ສິດ", "ຂອງ", "ມະນຸດ"],
            LineSegmenter::new_dictionary(Default::default()),
        );
    }

    #[test]
    fn lao_line_break_17() {
        check_line(
            "ກ່ຽວກັບສິດຂອງມະນຸດ",
            &["ກ່ຽວ", "ກັບ", "ສິດ", "ຂອງ", "ມະນຸດ"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "ກ່ຽວກັບສິດຂອງມະນຸດ",
            &["ກ່ຽວກັບ", "ສິດ", "ຂອງ", "ມະນຸດ"],
            {
                let mut s = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn lao_line_break_neo() {
        check_line(
            "ກ່ຽວກັບສິດຂອງມະນຸດ",
            &["ກ່ຽວ", "ກັບ", "ສິດ", "ຂອງ", "ມະນຸດ"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_lstm();
                s
            },
        );

        check_line(
            "ກ່ຽວກັບສິດຂອງມະນຸດ",
            &["ກ່ຽວກັບ", "ສິດ", "ຂອງ", "ມະນຸດ"],
            {
                let mut s = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
                s.load_dictionary();
                s
            },
        );
    }

    #[test]
    fn empty_string() {
        let segmenter = LineSegmenter::new_for_non_complex_scripts(Default::default());
        let breaks: Vec<usize> = segmenter.segment_str("").collect();
        assert_eq!(breaks, [0]);
    }

    #[test]
    fn empty_string_17() {
        let segmenter = LineSegmenter::new_17_for_non_complex_scripts(Default::default());
        let breaks: Vec<usize> = segmenter.segment_str("").collect();
        assert_eq!(breaks, [0]);
    }

    #[test]
    fn empty_string_neo() {
        let segmenter = LineSegmenter::new_neo_for_non_complex_scripts(Default::default());
        let breaks: Vec<usize> = segmenter.segment_str("").collect();
        assert_eq!(breaks, [0]);
    }
