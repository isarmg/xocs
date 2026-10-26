use blog_assets_compiler::{
    borrowed_text::{CowUtils, Pattern},
    browser_targets::{self, Error as BrowserTargetsError, Opts},
    byte_count,
};
use std::{borrow::Cow, panic::catch_unwind};

#[test]
fn browser_query_numbers_do_not_wrap_or_panic() {
    for query in [
        "last 65536 node versions",
        "last 65537 node versions",
        "last 999999999999999999999999 versions",
        "since 4294969320",
        "since 2147483648",
        "since -2147483648",
        "since -2147483649",
        "since 2024-4294967297-2",
        "since 2024-1-4294967297",
    ] {
        let result = catch_unwind(|| browser_targets::resolve(&[query], &Opts::default()));
        assert!(result.is_ok(), "panicked on {query}");
        assert!(result.unwrap().is_err(), "accepted {query}");
    }
    assert!(
        !browser_targets::resolve(&["last 65535 node versions"], &Opts::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn since_defaults_only_omitted_date_components() {
    let opts = Opts::default();
    let full = browser_targets::resolve(&["since 2024-1-1"], &opts).unwrap();
    assert!(!full.is_empty());
    for query in ["since 2024", "since +2024", "since 2024-1"] {
        assert_eq!(browser_targets::resolve(&[query], &opts).unwrap(), full);
    }
    for query in ["since 2024-", "since 2024--2", "since 2024-1-"] {
        assert!(
            browser_targets::resolve(&[query], &opts).is_err(),
            "accepted {query}"
        );
    }
}

#[test]
fn malformed_node_versions_are_rejected_without_zero_defaults() {
    use std::error::Error as _;

    for ignore_unknown_versions in [false, true] {
        let opts = Opts {
            ignore_unknown_versions,
            ..Opts::default()
        };
        for query in [
            "node 0.65536",
            "node 20.65536.0",
            "node 20.0.65536",
            "node 20..0",
            "node >= 65536",
        ] {
            let error = browser_targets::resolve(&[query], &opts).unwrap_err();
            assert!(matches!(
                error,
                BrowserTargetsError::InvalidNodejsVersion { .. }
            ));
            assert!(error.source().is_some());
        }
        assert!(browser_targets::resolve(&["node 20.0.0.0"], &opts).is_err());
    }
}

#[test]
fn optional_node_components_and_unknown_version_policy_are_preserved() {
    let opts = Opts::default();
    for query in ["node 20", "node 20.0", "node 20.0.0"] {
        let versions = browser_targets::resolve(&[query], &opts).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].name(), "node");
        assert!(versions[0].version().starts_with("20."));
    }
    assert!(matches!(
        browser_targets::resolve(&["node 65535"], &opts),
        Err(BrowserTargetsError::UnknownNodejsVersion(_))
    ));
    let opts = Opts {
        ignore_unknown_versions: true,
        ..Opts::default()
    };
    assert!(
        browser_targets::resolve(&["node 65535"], &opts)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn node_range_endpoints_reject_invalid_components_and_extra_segments() {
    use std::error::Error as _;

    for query in [
        "node 0.65536 - 20",
        "node 0 - 20.65536",
        "node 0.0.65536 - 20",
        "node 0 - 20..1",
        "node >= 20..0",
    ] {
        let error = browser_targets::resolve(&[query], &Opts::default()).unwrap_err();
        assert!(matches!(
            error,
            BrowserTargetsError::InvalidNodejsVersion { .. }
        ));
        assert!(error.source().is_some());
    }
    for query in ["node >= 20.0.0.0", "node 0.0.0.0 - 20", "node 0 - 20.0.0.0"] {
        assert!(
            browser_targets::resolve(&[query], &Opts::default()).is_err(),
            "accepted {query}"
        );
    }
}

#[test]
fn node_range_comparisons_keep_optional_component_semantics() {
    let opts = Opts::default();
    let bounded = browser_targets::resolve(&["node 20.0 - 20.1"], &opts).unwrap();
    assert!(!bounded.is_empty());
    for version in &bounded {
        assert!(version.version().starts_with("20.0.") || version.version().starts_with("20.1."));
    }
    assert_eq!(
        browser_targets::resolve(&["node 20.0.99 - 20.1.99"], &opts).unwrap(),
        bounded
    );
    assert_eq!(
        browser_targets::resolve(&["node >= 20"], &opts).unwrap(),
        browser_targets::resolve(&["node >= 20.0.0"], &opts).unwrap()
    );
}

struct CustomMatches(Vec<(usize, &'static str)>);

impl<'s> Pattern<'s> for CustomMatches {
    type MatchIndices = std::vec::IntoIter<(usize, &'s str)>;

    fn match_indices_in(self, _source: &'s str) -> Self::MatchIndices {
        self.0.into_iter()
    }
}

#[test]
fn safe_custom_patterns_cannot_bypass_string_boundaries() {
    for (source, matches) in [
        ("a", vec![(2, "")]),
        ("é", vec![(1, "")]),
        ("a", vec![(0, "longer than source")]),
        ("abc", vec![(2, "c"), (1, "b")]),
    ] {
        assert!(catch_unwind(|| source.cow_replace(CustomMatches(matches), "x")).is_err());
    }
}

#[test]
fn replacement_preserves_borrowing_and_unicode_matches() {
    assert!(matches!(
        "école".cow_replace('z', ""),
        Cow::Borrowed("école")
    ));
    assert_eq!("é-é-é".cow_replace('é', "字"), "字-字-字");
    assert_eq!("é-é-é".cow_replacen('é', "字", 2), "字-字-é");
    assert_eq!("é-é-é".cow_replacen('é', "字", 0), "é-é-é");
    assert_eq!("ab".cow_replace("", "/"), "/a/b/");
}

#[test]
fn native_word_scans_match_scalar_counts_at_alignment_and_chunk_boundaries() {
    let bytes = (0..4128)
        .map(|index| ((index * 37) % 256) as u8)
        .collect::<Vec<_>>();
    let lengths = (0..96).chain([2039, 2040, 2041, 2047, 2048, 2049, 4096]);
    for length in lengths {
        for offset in 0..16 {
            let slice = &bytes[offset..offset + length];
            for needle in [0, 1, 127, 128, 255] {
                assert_eq!(
                    byte_count::count(slice, needle),
                    slice.iter().filter(|&&value| value == needle).count(),
                    "length={length}, offset={offset}, needle={needle}"
                );
            }
            assert_eq!(
                byte_count::num_chars(slice),
                slice.iter().filter(|&&value| value & 0xc0 != 0x80).count(),
                "length={length}, offset={offset}"
            );
        }
    }
}
