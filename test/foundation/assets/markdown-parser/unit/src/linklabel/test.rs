
    use super::scan_link_label_rest;

    #[test]
    fn whitespace_normalization() {
        let input = "«\t\tBlurry Eyes\t\t»][blurry_eyes]";
        let expected_output = "« Blurry Eyes »"; // regular spaces!

        let (_bytes, normalized_label) = scan_link_label_rest(input, &|_| None, false).unwrap();
        assert_eq!(expected_output, normalized_label.as_ref());
    }

    #[test]
    fn return_carriage_linefeed_ok() {
        let input = "hello\r\nworld\r\n]";
        assert!(scan_link_label_rest(input, &|_| Some(0), false).is_some());
    }
