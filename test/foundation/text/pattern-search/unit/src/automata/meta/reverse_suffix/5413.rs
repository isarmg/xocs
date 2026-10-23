#[cfg(all(feature = "unicode-perl", not(miri),))]
#[cfg(test)]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../test/foundation/text/pattern-search/integration/retained/test/foundation/regex/automata/unit/src/meta/reverse_suffix/tests_173.rs"));
