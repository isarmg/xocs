
    #![expect(
        clippy::cast_possible_truncation,
        reason = "deterministic fuzz inputs are tightly bounded"
    )]

    use super::*;

    /// Deterministic xorshift so any failure reproduces from a fixed seed.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// Assert `SpanContents::line_column_at` matches a real
    /// `read_span(&(off, 0), 0, 0)` for an offset inside a
    /// `read_span(&(off, len), ctx, ctx)` payload — exactly how the graphical
    /// renderer consumes it. Full equivalence, including failure:
    /// `line_column_at` must return `None` iff the direct read returns `None`
    /// (a label past the snippet, e.g. `off == src.len() + 1`). Returns whether
    /// a case was actually checked (the context read succeeded).
    fn check(src: &str, off: usize, len: usize, ctx: usize) -> bool {
        let Some(contents) =
            SpanReader::from_span(src.as_bytes(), Span::sized(off as u32, len as u32), ctx, ctx)
                .read()
        else {
            return false;
        };
        let got = contents.line_column_at(off);

        match SpanReader::from_span(src.as_bytes(), Span::empty(off as u32), 0, 0).read() {
            Some(expected) => assert_eq!(
                got,
                Some((expected.line(), expected.column())),
                "mismatch for src={src:?} off={off} len={len} ctx={ctx}"
            ),
            None => assert_eq!(
                got, None,
                "accepted an out-of-bounds label for src={src:?} off={off} len={len} ctx={ctx}"
            ),
        }
        true
    }

    /// Exhaustively check every char-boundary offset of many small randomized
    /// sources — plus offsets one and two bytes past EOF — across LF / CRLF /
    /// lone-CR and multibyte alphabets.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "equivalence fuzzer over safe, bounds-checked code — Miri finds no UB here \
                  and interprets it orders of magnitude slower; the derivation is still exercised \
                  under Miri by the normal snapshot tests"
    )]
    fn matches_read_span_exhaustively() {
        let alphabets: &[&[&str]] = &[
            &["a", "\n"],
            &["a", "b", "c", "\n"],
            &["a", "b", "\r\n"],
            &["a", "\r", "\n", "\r\n"],
            &["x", "y", "\n", "é", "🦀", "\r\n"],
        ];
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut checked = 0usize;
        for alpha in alphabets {
            for _ in 0..3000 {
                let n = rng.below(14);
                let mut s = String::new();
                for _ in 0..n {
                    s.push_str(alpha[rng.below(alpha.len())]);
                }
                // Include `len + 1` / `len + 2`, which are past EOF (never char
                // boundaries) to exercise the out-of-bounds rejection.
                for off in 0..=s.len() + 2 {
                    if off <= s.len() && !s.is_char_boundary(off) {
                        continue;
                    }
                    for ctx in 0..=2 {
                        for &len in &[0usize, 1, 4] {
                            if check(&s, off, len, ctx) {
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(checked > 100_000, "expected a broad sweep, only checked {checked}");
    }
