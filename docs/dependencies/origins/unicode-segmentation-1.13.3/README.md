Iterators which split strings on Grapheme Cluster, Word, or Sentence boundaries,
according to the
[Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/) rules.

[![Build Status](https://github.com/unicode-rs/unicode-segmentation/actions/workflows/rust.yml/badge.svg)](https://github.com/unicode-rs/unicode-segmentation/actions/workflows/rust.yml)

[Documentation](https://docs.rs/unicode-segmentation)

```rust
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let s = "a̐éö̲\r\n";
    let g = s.graphemes(true).collect::<Vec<&str>>();
    let b: &[_] = &["a̐", "é", "ö̲", "\r\n"];
    assert_eq!(g, b);

    let s = "The quick (\"brown\") fox can't jump 32.3 feet, right?";
    let w = s.unicode_words().collect::<Vec<&str>>();
    let b: &[_] = &["The", "quick", "brown", "fox", "can't", "jump", "32.3", "feet", "right"];
    assert_eq!(w, b);

    let s = "The quick (\"brown\")  fox";
    let w = s.split_word_bounds().collect::<Vec<&str>>();
    let b: &[_] = &["The", " ", "quick", " ", "(", "\"", "brown", "\"", ")", "  ", "fox"];
    assert_eq!(w, b);
}
```

# no_std

unicode-segmentation does not depend on libstd, so it can be used in crates
with the `#![no_std]` attribute.

# crates.io

You can use this package in your project by adding the following
to your `Cargo.toml`:

```toml
[dependencies]
unicode-segmentation = "1"
```
