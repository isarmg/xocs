//! Implements the [`proptest::arbitrary::Arbitrary`] trait for [`CompactString`]

use alloc::string::String;

use proptest::arbitrary::StrategyFor;
use proptest::prelude::*;
use proptest::strategy::MapInto;
use proptest::string::StringParam;

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "proptest")))]
impl Arbitrary for CompactString {
    type Parameters = StringParam;
    type Strategy = MapInto<StrategyFor<String>, Self>;

    fn arbitrary_with(a: Self::Parameters) -> Self::Strategy {
        any_with::<String>(a).prop_map_into()
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/proptest/test.rs"]
#[cfg(test)]
mod test;
