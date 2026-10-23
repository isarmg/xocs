#[cfg(all(
    feature = "perf-literal-substring",
    feature = "perf-literal-multisubstring",
    feature = "unicode-perl",
    not(miri),
))]
#[cfg(test)]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../test/foundation/text/pattern-search/integration/retained/test/foundation/regex/automata/unit/src/meta/strategy/tests_1958.rs"));
