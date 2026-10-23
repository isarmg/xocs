use blog_assets_compiler::{
    borrowed_text::{CowUtils, Pattern},
    byte_count,
};
use std::{borrow::Cow, panic::catch_unwind};

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
