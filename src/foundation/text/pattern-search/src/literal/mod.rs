/*!
Multi-pattern substring search using Aho-Corasick, enabled by `literal`.

[`AhoCorasick`](crate::literal::AhoCorasick) searches compiled patterns;
[`AhoCorasickBuilder`](crate::literal::AhoCorasickBuilder) configures the
construction and match semantics. [`MatchKind`](crate::literal::MatchKind)
selects standard, leftmost-first or leftmost-longest matches. Overlapping
search, replacements and stream methods each document the supported match
kind and resource requirements.

```
use blog_text_pattern_search::literal::{AhoCorasick, PatternID};
let matcher = AhoCorasick::new(["apple", "maple"])?;
let found = matcher.find("maple").unwrap();
assert_eq!(found.pattern(), PatternID::must(1));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The [`automaton`](crate::literal::automaton) module exposes the lower-level
transition interface. [`dfa`](crate::literal::dfa) and
[`nfa`](crate::literal::nfa) provide automaton representations;
[`packed`](crate::literal::packed) provides packed search for smaller pattern
sets. Builders choose suitable implementations by default.

Stream methods require `std`. Literal prefilters can use the local byte-search
implementation through the `literal-perf` feature. Configure pattern counts,
input sizes and memory limits at the consuming application boundary.
*/

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;


#[cfg(feature = "std")]
pub use crate::literal::ahocorasick::StreamFindIter;
pub use crate::literal::{
    ahocorasick::{
        AhoCorasick, AhoCorasickBuilder, AhoCorasickKind, FindIter,
        FindOverlappingIter,
    },
    util::{
        error::{BuildError, MatchError, MatchErrorKind},
        primitives::{PatternID, PatternIDError},
        search::{Anchored, Input, Match, MatchKind, Span, StartKind},
    },
};

#[macro_use]
mod macros;

mod ahocorasick;
pub mod automaton;
pub mod dfa;
pub mod nfa;
pub mod packed;
#[path = "../../../../../../test/foundation/text/pattern-search/unit/src/literal/mod/tests.rs"]
#[cfg(test)]
mod tests;
// I wrote out the module for implementing fst::Automaton only to later realize
// that this would make fst a public dependency and fst is not at 1.0 yet. I
// decided to just keep the code in tree, but build it only during tests.
//
// TODO: I think I've changed my mind again. I'm considering pushing it out
// into either a separate crate or into 'fst' directly as an optional feature.
// #[cfg(test)]
// #[allow(dead_code)]
// mod transducer;
pub(crate) mod util;

#[cfg(test)]
include!("../../../../../../test/foundation/text/pattern-search/unit/src/literal/mod/2622.rs");
