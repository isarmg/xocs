
    use super::*;

    #[test]
    fn decode_sourcemap() {
        let input = r#"{
            "version": 3,
            "sources": ["coolstuff.js"],
            "sourceRoot": "x",
            "names": ["x","alert"],
            "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM",
            "ignoreList": [0]
        }"#;
        let sm = SourceMap::from_json_string(input).unwrap();
        assert_eq!(sm.get_source_root(), Some("x"));
        assert_eq!(sm.get_ignore_list(), Some(&[0][..]));
        let mut iter = sm.get_source_view_tokens().filter(|token| token.get_name_id().is_some());
        assert_eq!(iter.next().unwrap().to_tuple(), (Some("coolstuff.js"), 0, 4, Some("x")));
        assert_eq!(iter.next().unwrap().to_tuple(), (Some("coolstuff.js"), 1, 4, Some("x")));
        assert_eq!(iter.next().unwrap().to_tuple(), (Some("coolstuff.js"), 2, 2, Some("alert")));
        assert!(iter.next().is_none());
    }

    #[test]
    fn decode_from_json_value() {
        // `SourceMap::from_json` / `decode` consumes an owned `JSONSourceMap`,
        // a separate path from the borrowed `from_json_string` / `decode_from_string`.
        let json = serde_json::from_value(serde_json::json!({
            "version": 3,
            "file": "f.js",
            "sourceRoot": "r",
            "names": ["n"],
            "sources": ["a.js"],
            "sourcesContent": ["c"],
            "mappings": "AAAAA",
            "debugId": "d",
            "ignoreList": [0],
            "x_google_ignoreList": false
        }))
        .unwrap();
        let sm = SourceMap::from_json(json).unwrap();
        assert_eq!(sm.get_file(), Some("f.js"));
        assert_eq!(sm.get_source_root(), Some("r"));
        assert_eq!(sm.get_debug_id(), Some("d"));
        assert_eq!(sm.get_ignore_list(), Some(&[0][..]));
        assert_eq!(sm.get_source_content(0), Some("c"));
        assert_eq!(sm.get_name(0), Some("n"));
    }

    #[test]
    fn decode_ignore_list_fallback() {
        let cases: &[(&str, Option<&[u32]>)] = &[
            ("", None),
            (r#", "ignoreList": [0]"#, Some(&[0])),
            (r#", "x_google_ignoreList": [1]"#, Some(&[1])),
            (r#", "ignoreList": [0], "x_google_ignoreList": [0]"#, Some(&[0])),
            (r#", "ignoreList": [0], "x_google_ignoreList": [1]"#, Some(&[0])),
            (r#", "x_google_ignoreList": [1], "ignoreList": [0]"#, Some(&[0])),
            (r#", "ignoreList": [], "x_google_ignoreList": [1]"#, Some(&[])),
            (r#", "ignoreList": [0], "x_google_ignoreList": [3]"#, Some(&[0])),
            (r#", "ignoreList": [], "x_google_ignoreList": [3]"#, Some(&[])),
            (r#", "ignoreList": [0], "x_google_ignoreList": false"#, Some(&[0])),
            (r#", "x_google_ignoreList": false, "ignoreList": [0]"#, Some(&[0])),
            (r#", "x_google_ignoreList": false, "ignoreList": []"#, Some(&[])),
            (r#", "ignoreList": [0], "x_google_ignoreList": [-1]"#, Some(&[0])),
            (r#", "ignoreList": null, "x_google_ignoreList": [1]"#, Some(&[1])),
            (r#", "ignoreList": [0], "x_google_ignoreList": null"#, Some(&[0])),
            (r#", "ignoreList": null, "x_google_ignoreList": null"#, None),
            (r#", "ignoreList": null"#, None),
            (r#", "x_google_ignoreList": null"#, None),
            (r#", "ignoreList": []"#, Some(&[])),
            (r#", "x_google_ignoreList": []"#, Some(&[])),
        ];
        for &(fields, expected) in cases {
            let input =
                format!(r#"{{"version":3,"sources":["a.js","b.js"],"mappings":""{fields}}}"#);
            let borrowed = SourceMap::from_json_string(&input).unwrap();
            assert_eq!(borrowed.get_ignore_list(), expected, "{input}");

            let json = serde_json::from_reader(input.as_bytes()).unwrap();
            let owned = SourceMap::from_json(json).unwrap();
            assert_eq!(owned.get_ignore_list(), expected, "{input}");
        }
    }

    #[test]
    fn decode_ignore_list_invalid_types() {
        for fields in [
            r#""ignoreList": false"#,
            r#""ignoreList": [-1]"#,
            r#""ignoreList": [0.5]"#,
            r#""ignoreList": [4294967296]"#,
            r#""x_google_ignoreList": false"#,
            r#""x_google_ignoreList": [-1]"#,
            r#""ignoreList": false, "x_google_ignoreList": [0]"#,
            r#""ignoreList": null, "x_google_ignoreList": false"#,
            r#""ignoreList": null, "x_google_ignoreList": [-1]"#,
        ] {
            let input = format!(r#"{{"version":3,"sources":["a.js"],"mappings":"",{fields}}}"#);
            assert!(
                matches!(SourceMap::from_json_string(&input), Err(Error::BadJson(_))),
                "{input}"
            );
            assert!(serde_json::from_str::<JSONSourceMap>(&input).is_err(), "{input}");
        }
    }

    #[test]
    fn decode_sourcemap_optional_field() {
        let input = r#"{
            "version": 3,
            "names": [],
            "sources": [],
            "sourcesContent": [null],
            "mappings": ""
        }"#;
        SourceMap::from_json_string(input).expect("should success");
    }

    #[test]
    fn decode_unsupported_version() {
        let input = r#"{"version": 2, "names": [], "sources": [], "mappings": ""}"#;
        let err = SourceMap::from_json_string(input).unwrap_err();
        assert!(matches!(err, Error::BadJson(_)));
    }

    #[test]
    fn decode_mapping_bad_segment_size() {
        let input = r#"{"version":3,"names":[],"sources":[],"sourcesContent":[],"mappings":"AA"}"#;
        let err = SourceMap::from_json_string(input).unwrap_err();
        assert!(matches!(err, Error::BadSegmentSize(2)));
    }

    #[test]
    fn decode_mapping_vlq_leftover() {
        let input = r#"{"version":3,"names":[],"sources":[],"sourcesContent":[],"mappings":"g"}"#;
        let err = SourceMap::from_json_string(input).unwrap_err();
        assert!(matches!(err, Error::VlqLeftover));
    }

    #[test]
    fn decode_mapping_vlq_overflow() {
        // A long run of continuation bytes overflows the VLQ shift accumulator.
        let mappings = "g".repeat(14);
        let input =
            format!(r#"{{"version":3,"names":[],"sources":["a.js"],"mappings":"{mappings}"}}"#);
        let err = SourceMap::from_json_string(&input).unwrap_err();
        assert!(matches!(err, Error::VlqOverflow));
    }

    #[test]
    fn decode_mapping_bad_source_reference() {
        // 4-field segment references source id 0, but there are no sources.
        let input = r#"{"version":3,"names":[],"sources":[],"mappings":"AAAA"}"#;
        let err = SourceMap::from_json_string(input).unwrap_err();
        assert!(matches!(err, Error::BadSourceReference(_)));
    }

    #[test]
    fn decode_mapping_bad_name_reference() {
        // 5-field segment references name id 0, but there are no names.
        let input = r#"{"version":3,"names":[],"sources":["a.js"],"mappings":"AAAAA"}"#;
        let err = SourceMap::from_json_string(input).unwrap_err();
        assert!(matches!(err, Error::BadNameReference(0)));
    }

    #[test]
    fn decode_ignore_list_bad_source_reference() {
        for fields in [
            r#""ignoreList": [3]"#,
            r#""x_google_ignoreList": [3]"#,
            r#""ignoreList": [3], "x_google_ignoreList": [0]"#,
            r#""ignoreList": null, "x_google_ignoreList": [3]"#,
        ] {
            let input = format!(r#"{{"version":3,"sources":["a.js"],"mappings":"",{fields}}}"#);
            assert!(
                matches!(SourceMap::from_json_string(&input), Err(Error::BadSourceReference(3))),
                "{input}"
            );
            let json = serde_json::from_str(&input).unwrap();
            assert!(
                matches!(SourceMap::from_json(json), Err(Error::BadSourceReference(3))),
                "{input}"
            );
        }
    }

    #[test]
    fn decode_owned_propagates_errors() {
        let empty = || SourceMap::default().to_json();
        // The owned `decode` path (`SourceMap::from_json`) must surface the same
        // validation errors as the borrowed path: an unsupported version...
        let bad_version = JSONSourceMap { version: 4, ..empty() };
        assert!(matches!(SourceMap::from_json(bad_version), Err(Error::BadJson(_))));

        // ...a bad ignore-list index...
        let bad_ignore_list = JSONSourceMap { ignore_list: Some(vec![3]), ..empty() };
        assert!(matches!(SourceMap::from_json(bad_ignore_list), Err(Error::BadSourceReference(3))));

        // ...and a mapping that references a source with no sources declared.
        let bad_mapping = JSONSourceMap { mappings: "AAAA".to_string(), ..empty() };
        assert!(matches!(SourceMap::from_json(bad_mapping), Err(Error::BadSourceReference(_))));
    }

    #[test]
    fn decode_invalid_char_behaves_like_slash() {
        // Bytes outside the base64 alphabet decode as payload 31 with the
        // continuation bit set — exactly like '/'. A map using '!' must
        // therefore produce the same tokens as the same map using '/'.
        let make = |mappings: &str| {
            format!(r#"{{"version":3,"names":[],"sources":[],"mappings":"{mappings}"}}"#)
        };
        let bang_json = make("gD,!C");
        let slash_json = make("gD,/C");
        let bang = SourceMap::from_json_string(&bang_json).unwrap();
        let slash = SourceMap::from_json_string(&slash_json).unwrap();
        let tokens: Vec<Token> = bang.get_tokens().collect();
        assert_eq!(tokens, slash.get_tokens().collect::<Vec<Token>>());
        assert_eq!(tokens[0].get_dst_col(), 48);
        assert_eq!(tokens[1].get_dst_col(), 1);
    }

    #[test]
    fn decode_dangling_continuation_before_delimiter_is_leftover() {
        // 13 continuation bytes stay under the shift-overflow bound, so ending
        // the segment there must surface VlqLeftover, not VlqOverflow.
        let mappings = format!("{},A", "g".repeat(13));
        let input = format!(r#"{{"version":3,"names":[],"sources":[],"mappings":"{mappings}"}}"#);
        let err = SourceMap::from_json_string(&input).unwrap_err();
        assert!(matches!(err, Error::VlqLeftover));
    }

    #[test]
    fn decode_oversized_segments_report_exact_field_count() {
        for (mappings, expected) in [("AAAAAA", 6u32), ("AAAAAAA", 7u32)] {
            let input =
                format!(r#"{{"version":3,"names":[],"sources":[],"mappings":"{mappings}"}}"#);
            let err = SourceMap::from_json_string(&input).unwrap_err();
            assert!(matches!(err, Error::BadSegmentSize(n) if n == expected));
        }
    }

    #[test]
    fn decode_parse_error_beats_segment_validation() {
        // The whole segment is VLQ-parsed before any field validation, so a
        // dangling continuation reports VlqLeftover even when the decoded
        // fields would also fail the column check ("Dg": delta -1) or the
        // segment-size check ("AAAAAg": 6 fields).
        for mappings in ["Dg", "AAAAAg"] {
            let input =
                format!(r#"{{"version":3,"names":[],"sources":[],"mappings":"{mappings}"}}"#);
            let err = SourceMap::from_json_string(&input).unwrap_err();
            assert!(matches!(err, Error::VlqLeftover), "mappings {mappings:?}: {err:?}");
        }
    }

    #[test]
    fn decode_negative_column_beats_size_check() {
        // "DA" decodes to two fields with a negative generated column; the
        // column check runs before the field-count check.
        let input = r#"{"version":3,"names":[],"sources":[],"mappings":"DA"}"#;
        let err = SourceMap::from_json_string(input).unwrap_err();
        assert!(matches!(err, Error::BadSegmentSize(0)));
    }

    #[test]
    fn parse_vlq_segment_empty_is_no_values() {
        // Directly exercise the defensive `VlqNoValues` branch: with the cursor
        // already at the end of the input, no values can be decoded. This state
        // is unreachable through `decode_mapping` (which only calls the parser
        // on a non-delimiter byte), so it is covered here.
        let mut cursor = 0;
        let mut out = [0i64; 5];
        let err = parse_vlq_segment_into(b"", &mut cursor, &mut out).unwrap_err();
        assert!(matches!(err, Error::VlqNoValues));
    }
