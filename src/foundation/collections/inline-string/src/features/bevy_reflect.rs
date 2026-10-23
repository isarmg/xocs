use crate::CompactString;
use bevy_reflect::{
    impl_reflect_opaque, std_traits::ReflectDefault, ReflectDeserialize, ReflectSerialize,
};

impl_reflect_opaque!((in crate::CompactString)CompactString(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
));

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/bevy_reflect/tests.rs"]
#[cfg(test)]
mod tests;
