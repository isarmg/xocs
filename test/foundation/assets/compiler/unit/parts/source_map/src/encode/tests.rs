
    use super::*;

    #[test]
    fn encode_empty_ignore_list() {
        let mut sm = SourceMap::default();
        sm.set_ignore_list(vec![]);
        assert_eq!(
            sm.to_json_string(),
            r#"{"version":3,"names":[],"sources":[],"ignoreList":[],"mappings":""}"#
        );
    }

    #[test]
    fn encode_ignore_list() {
        for fields in [
            "",
            r#", "ignoreList": []"#,
            r#", "ignoreList": [0]"#,
            r#", "x_google_ignoreList": []"#,
            r#", "x_google_ignoreList": [0]"#,
            r#", "ignoreList": [], "x_google_ignoreList": [0]"#,
        ] {
            let input = format!(r#"{{"version":3,"sources":["a.js"],"mappings":""{fields}}}"#);
            let sm = SourceMap::from_json_string(&input).unwrap();
            assert_eq!(sm.to_json().ignore_list.as_deref(), sm.get_ignore_list());

            let encoded = sm.to_json_string();
            let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
            assert!(json.get("x_google_ignoreList").is_none());
            assert_eq!(
                json.get("ignoreList"),
                sm.get_ignore_list().map(|list| serde_json::json!(list)).as_ref()
            );
            let decoded = SourceMap::from_json_string(&encoded).unwrap();
            assert_eq!(decoded.get_ignore_list(), sm.get_ignore_list());
        }
    }

    #[test]
    fn encode_roundtrip() {
        let input = r#"{
        "version": 3,
        "sources": ["coolstuff.js"],
        "sourceRoot": "x",
        "names": ["x","alert"],
        "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM",
        "ignoreList": [0]
    }"#;
        let sm = SourceMap::from_json_string(input).unwrap();
        let encoded = sm.to_json_string();
        let sm2 = SourceMap::from_json_string(&encoded).unwrap();

        for (tok1, tok2) in sm.get_tokens().zip(sm2.get_tokens()) {
            assert_eq!(tok1, tok2);
        }

        // spellchecker:off
        let input = r#"{
        "version": 3,
        "file": "index.js",
        "names": [
            "text",
            "text"
        ],
        "sources": [
            "../../hmr.js",
            "../../main.js",
            "../../index.html"
        ],
        "sourcesContent": [
            "export const foo = 'hello'\n\ntext('.hmr', foo)\n\nfunction text(el, text) {\n  document.querySelector(el).textContent = text\n}\n\nimport.meta.hot?.accept((mod) =\u003E {\n  if (mod) {\n    text('.hmr', mod.foo)\n  }\n})\n",
            "import './hmr.js'\n\ntext('.app', 'hello')\n\nfunction text(el, text) {\n  document.querySelector(el).textContent = text\n}\n",
            "\u003Ch1\u003EHMR Full Bundle Mode\u003C/h1\u003E\n\n\u003Cdiv class=\"app\"\u003E\u003C/div\u003E\n\u003Cdiv class=\"hmr\"\u003E\u003C/div\u003E\n\n\u003Cscript type=\"module\" src=\"./main.js\"\u003E\u003C/script\u003E\n"
        ],
        "mappings": ";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;AAAA,MAAa,MAAM;AAEnBA,OAAK,QAAQ,IAAI;AAEjB,SAASA,OAAK,IAAI,QAAM;AACtB,UAAS,cAAc,GAAG,CAAC,cAAcA;;SAG1B,QAAQ,QAAQ;AAC/B,KAAI,KAAK;AACP,SAAK,QAAQ,IAAI,IAAI;;EAEvB;;;;;;;ACVF,KAAK,QAAQ,QAAQ;AAErB,SAAS,KAAK,IAAI,QAAM;AACtB,UAAS,cAAc,GAAG,CAAC,cAAcC"
    }"#;
        // spellchecker:on
        let sm = SourceMap::from_json_string(input).unwrap();
        let encoded = sm.to_json_string();
        let sm2 = SourceMap::from_json_string(&encoded).unwrap();

        for (tok1, tok2) in sm.get_tokens().zip(sm2.get_tokens()) {
            assert_eq!(tok1, tok2);
        }
    }

    #[test]
    fn encode_escape_string() {
        // '\0' should be escaped.
        let mut sm = SourceMap::new(
            None,
            vec!["name_length_greater_than_16_\0".into()],
            None,
            vec!["\0".into()],
            vec![Some("emoji-👀-\0".into())],
            vec![].into_boxed_slice(),
            None,
        );
        sm.set_ignore_list(vec![0]);
        sm.set_debug_id("56431d54-c0a6-451d-8ea2-ba5de5d8ca2e");
        assert_eq!(
            sm.to_json_string(),
            r#"{"version":3,"names":["name_length_greater_than_16_\u0000"],"sources":["\u0000"],"sourcesContent":["emoji-👀-\u0000"],"ignoreList":[0],"mappings":"","debugId":"56431d54-c0a6-451d-8ea2-ba5de5d8ca2e"}"#
        );
    }

    #[test]
    fn vlq_encode_diff() {
        // Most important tests here are that with maximum values, `encode_vlq` pushes maximum of 7 bytes.
        // This invariant is essential to safety of `encode_vlq`.
        #[rustfmt::skip]
    const FIXTURES: &[(u32, u32, &str)] = &[
        (0,           0, "A"),
        (1,           0, "C"),
        (2,           0, "E"),
        (15,          0, "e"),
        (16,          0, "gB"),
        (511,         0, "+f"),
        (512,         0, "ggB"),
        (16_383,      0, "+/f"),
        (16_384,      0, "gggB"),
        (524_287,     0, "+//f"),
        (524_288,     0, "ggggB"),
        (16_777_215,  0, "+///f"),
        (16_777_216,  0, "gggggB"),
        (536_870_911, 0, "+////f"),
        (536_870_912, 0, "ggggggB"),
        (u32::MAX,    0, "+/////H"), // 7 bytes

        (0, 1,           "D"),
        (0, 2,           "F"),
        (0, 15,          "f"),
        (0, 16,          "hB"),
        (0, 511,         "/f"),
        (0, 512,         "hgB"),
        (0, 16_383,      "//f"),
        (0, 16_384,      "hggB"),
        (0, 524_287,     "///f"),
        (0, 524_288,     "hgggB"),
        (0, 16_777_215,  "////f"),
        (0, 16_777_216,  "hggggB"),
        (0, 536_870_911, "/////f"),
        (0, 536_870_912, "hgggggB"),
        (0, u32::MAX,    "//////H"), // 7 bytes
    ];

        for (a, b, res) in FIXTURES.iter().copied() {
            let mut out = String::with_capacity(MAX_VLQ_BYTES);
            // SAFETY: `out` has 7 bytes spare capacity
            unsafe { encode_vlq(&mut out, vlq_value(a, b)) };
            assert_eq!(&out, res);
        }
    }

    #[test]
    fn encode_fast_4_or_5_segment() {
        // Exercises the packed single-store fast path and its boundary with the
        // per-digit slow path: deltas of ±15 are single-digit (fast), ±16 are
        // not (slow), and negative deltas must keep the sign in the VLQ LSB.
        let tokens = vec![
            Token::new(0, 15, 15, 15, Some(0), Some(0)), // all-small, fast (5 fields)
            Token::new(0, 30, 0, 0, Some(0), None),      // negative deltas, fast (4 fields)
            Token::new(0, 46, 16, 16, Some(0), Some(0)), // +16 src deltas, slow
            Token::new(0, 47, 0, 0, Some(0), Some(0)),   // -16 src deltas, slow
            Token::new(1, 1, 1, 1, Some(0), Some(0)),    // after line break, fast
            Token::new(1, 2, 1, 1, None, None),          // no source, dst_col only
        ];
        let sm = SourceMap::new(
            None,
            vec!["a".into()],
            None,
            vec!["a.js".into()],
            vec![],
            tokens.into_boxed_slice(),
            None,
        );
        assert_eq!(sm.to_json().mappings, "eAeeA,eAff,gBAgBgBA,CAhBhBA;CACCA,C"); // spellchecker:disable-line
        let encoded = sm.to_json_string();
        let reparsed = SourceMap::from_json_string(&encoded).unwrap();
        assert!(sm.get_tokens().eq(reparsed.get_tokens()));
    }

    #[test]
    fn encode_all_sources_content_null() {
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec!["a.js".into(), "b.js".into()],
            vec![None, None],
            vec![].into_boxed_slice(),
            None,
        );
        let json = sm.to_json_string();
        assert!(
            !json.contains("sourcesContent"),
            "sourcesContent should be omitted when all items are None"
        );

        let json_map = encode(&sm);
        assert!(json_map.sources_content.is_none());

        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec!["a.js".into(), "b.js".into()],
            vec![Some("content".into()), None],
            vec![].into_boxed_slice(),
            None,
        );
        let json = sm.to_json_string();
        assert!(
            json.contains("sourcesContent"),
            "sourcesContent should be present when at least one item is Some"
        );
        assert!(
            json.contains(r#""sourcesContent":["content",null]"#),
            "None source_contents should be encoded as raw null, not quoted \"null\""
        );

        let json_map = encode(&sm);
        assert!(json_map.sources_content.is_some());
    }

    #[test]
    fn encode_escape_file_and_source_root() {
        let sm = SourceMap::new(
            Some("file\0name.js".into()),
            vec![],
            Some("root\0path".into()),
            vec![],
            vec![],
            vec![].into_boxed_slice(),
            None,
        );
        let json = sm.to_json_string();
        assert!(
            json.contains(r#""file":"file\u0000name.js""#),
            "file field should have \\0 escaped: {json}"
        );
        assert!(
            json.contains(r#""sourceRoot":"root\u0000path""#),
            "sourceRoot field should have \\0 escaped: {json}"
        );
        // Verify the output is valid JSON by round-tripping
        SourceMap::from_json_string(&json).unwrap();
    }

    #[test]
    fn encode_escape_debug_id() {
        let mut sm = SourceMap::default();
        // A debug_id containing JSON-special characters must be escaped, otherwise
        // the output is malformed JSON.
        sm.set_debug_id("id-with-\"quote\"-and-\\backslash");
        let json = sm.to_json_string();
        // Round-trip must succeed (would fail if the quote isn't escaped).
        let roundtripped = SourceMap::from_json_string(&json).unwrap();
        assert_eq!(roundtripped.get_debug_id(), Some("id-with-\"quote\"-and-\\backslash"));
    }

    #[test]
    fn encode_reserves_when_estimate_is_tight() {
        // `to_json` pre-sizes the mappings buffer at ~12 bytes/token. Several
        // tokens on the *same* generated line with large deltas each need more
        // than that, forcing the in-loop `reserve` (comma branch) to run.
        // Odd-indexed tokens carry no name id so the source-without-name
        // serialization branch is exercised too.
        let tokens: Vec<Token> = (0..32u32)
            .map(|i| {
                let name = if i % 2 == 0 { Some(0) } else { None };
                let source_position = (i % 2) * 536_870_912;
                Token::new(0, i * 100_000_000, source_position, source_position, Some(0), name)
            })
            .collect();
        let sm = SourceMap::new(
            None,
            vec!["n".into()],
            None,
            vec!["a.js".into()],
            vec![],
            tokens.into_boxed_slice(),
            None,
        );

        // Both encoders must round-trip the same tokens despite the realloc.
        let encoded = sm.to_json_string();
        let reparsed = SourceMap::from_json_string(&encoded).unwrap();
        assert!(sm.get_tokens().eq(reparsed.get_tokens()));
        let reparsed = SourceMap::from_json(sm.to_json()).unwrap();
        assert!(sm.get_tokens().eq(reparsed.get_tokens()));
    }

    #[test]
    fn encode_first_token_with_max_deltas() {
        // The first token of the first chunk emits no `;`/`,` delimiter, so it
        // must still trigger the per-token capacity check: its fields alone
        // can need up to 35 bytes while `to_json`'s estimate reserves ~12 per
        // token. src_line/src_col are unbounded by any array length.
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec!["a.js".into()],
            vec![],
            vec![Token::new(0, 536_870_912, 536_870_912, 536_870_912, Some(0), None)]
                .into_boxed_slice(),
            None,
        );
        let json = sm.to_json();
        assert_eq!(json.mappings, "ggggggBAggggggBggggggB");
        let json_string = sm.to_json_string();
        let reparsed = SourceMap::from_json_string(&json_string).unwrap();
        assert!(sm.get_tokens().eq(reparsed.get_tokens()));
    }

    #[test]
    fn encode_multiline_reserves_on_line_breaks() {
        // Tokens spread across many generated lines drive the semicolon
        // (line-break) reserve branch with a tight `to_json` estimate.
        let tokens: Vec<Token> = (0..8u32)
            .map(|i| Token::new(i * 3, 50_000, 50_000, 50_000, Some(0), Some(0)))
            .collect();
        let sm = SourceMap::new(
            None,
            vec!["name".into()],
            None,
            vec!["a.js".into()],
            vec![],
            tokens.into_boxed_slice(),
            None,
        );
        let reparsed = SourceMap::from_json(sm.to_json()).unwrap();
        assert!(sm.get_tokens().eq(reparsed.get_tokens()));
    }

    #[test]
    fn encode_tokens_without_source() {
        // Generated-column-only mappings (no source id) skip the source/name
        // VLQ fields entirely — the `source_id == None` serialization branch.
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![],
            vec![],
            vec![Token::new(0, 0, 0, 0, None, None), Token::new(0, 4, 0, 0, None, None)]
                .into_boxed_slice(),
            None,
        );
        let encoded = sm.to_json_string();
        let reparsed = SourceMap::from_json_string(&encoded).unwrap();
        assert!(sm.get_tokens().eq(reparsed.get_tokens()));
        assert!(reparsed.get_tokens().all(|token| token.get_source_id().is_none()));
    }
