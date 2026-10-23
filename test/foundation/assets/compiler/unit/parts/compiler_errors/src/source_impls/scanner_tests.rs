
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

    /// Run one query against a (stateful) scanner and assert the result is
    /// identical to a fresh `SpanReader` — data, span, line, column, and
    /// `line_count` on success, `None` on failure. `history` is the
    /// queries already issued to this scanner, which its state (and therefore
    /// any failure) depends on.
    fn check(
        scanner: &mut SpanScanner<'_>,
        input: &[u8],
        span: (usize, usize),
        before: usize,
        after: usize,
        history: &[(usize, usize)],
    ) {
        let source_span = Span::sized(span.0 as u32, span.1 as u32);
        let expected =
            SpanReader::new(input, SpanRequest::new(source_span), ContextLines::new(before, after))
                .read();
        let got = scanner.read_span(source_span);
        let src = String::from_utf8_lossy(input);
        match (&expected, &got) {
            (Some(expected), Some(got)) => {
                let fields =
                    |c: &SpanContents<'_>| (*c.span(), c.line(), c.column(), c.line_count());
                assert_eq!(
                    (fields(expected), expected.data()),
                    (fields(got), got.data()),
                    "mismatch for src={src:?} span={span:?} before={before} after={after} \
                     history={history:?}"
                );
            }
            (None, None) => {}
            _ => panic!(
                "expected {expected:?}, got {got:?} for src={src:?} span={span:?} \
                 before={before} after={after} history={history:?}"
            ),
        }
    }

    /// Differentially fuzz `SpanScanner` against `SpanReader` with query
    /// *sequences* — the scanner's whole point is state carried between
    /// queries, so single-shot checks would miss its interesting bugs. Covers
    /// LF / CRLF / lone-CR and multibyte sources; spans of any alignment
    /// (byte offsets, not char boundaries — neither function cares) including
    /// past-EOF ones; queries in random order (which exercises the
    /// out-of-index fallbacks) and in the renderer's sorted-then-merge order.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "equivalence fuzzer over safe, bounds-checked code — Miri finds no UB here \
                  and interprets it orders of magnitude slower; the scanner path is still \
                  exercised under Miri by the normal snapshot tests"
    )]
    fn scanner_matches_span_reader_exhaustively() {
        let alphabets: &[&[&str]] = &[
            &["a", "\n"],
            &["a", "b", "c", "\n"],
            &["a", "b", "\r\n"],
            &["a", "\r", "\n", "\r\n"],
            &["x", "y", "\n", "é", "🦀", "\r\n"],
        ];
        let mut rng = Rng(0xA076_1D64_78BD_642F);
        let mut checked = 0usize;
        for alpha in alphabets {
            for _ in 0..700 {
                let n = rng.below(16);
                let mut s = String::new();
                for _ in 0..n {
                    s.push_str(alpha[rng.below(alpha.len())]);
                }
                let input = s.as_bytes();
                for (before, after) in [(0, 0), (1, 1), (2, 2), (0, 2), (2, 0)] {
                    let mut spans: Vec<(usize, usize)> = std::iter::repeat_with(|| {
                        (rng.below(input.len() + 3), [0, 1, 2, 5][rng.below(4)])
                    })
                    .take(6)
                    .collect();

                    // Random order: queries may jump backwards past the
                    // index origin.
                    let mut scanner = SpanScanner::new(input, before, after);
                    for i in 0..spans.len() {
                        check(&mut scanner, input, spans[i], before, after, &spans[..i]);
                        checked += 1;
                    }

                    // The renderer's order: labels sorted by offset, then a
                    // merge attempt spanning from the first label to the
                    // furthest end.
                    spans.sort_unstable();
                    let first = spans[0].0;
                    let merged_end = spans.iter().map(|&(o, l)| o + l).max().unwrap();
                    spans.push((first, merged_end - first));
                    let mut scanner = SpanScanner::new(input, before, after);
                    for i in 0..spans.len() {
                        check(&mut scanner, input, spans[i], before, after, &spans[..i]);
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 100_000, "expected a broad sweep, only checked {checked}");
    }

    /// The empty-source and just-past-EOF edge cases `SpanReader` special
    /// cases, issued through one scanner.
    #[test]
    fn zero_length_spans_at_eof() {
        let mut scanner = SpanScanner::new(b"", 0, 0);
        check(&mut scanner, b"", (0, 0), 0, 0, &[]);
        check(&mut scanner, b"", (1, 0), 0, 0, &[(0, 0)]);
        let mut scanner = SpanScanner::new(b"a", 1, 1);
        check(&mut scanner, b"a", (1, 0), 1, 1, &[]);
        check(&mut scanner, b"a", (2, 0), 1, 1, &[(1, 0)]);
    }
