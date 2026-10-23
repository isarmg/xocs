#![allow(clippy::disallowed_names)]

#[path = "arrayvec.rs"]
#[cfg(feature = "arrayvec07")]
mod arrayvec;
#[path = "attr_order.rs"]
mod attr_order;
#[path = "bound.rs"]
mod bound;
#[path = "bytes.rs"]
#[cfg(feature = "bytes1")]
mod bytes;
#[path = "chrono.rs"]
#[cfg(feature = "chrono04")]
mod chrono;
#[path = "contract.rs"]
mod contract;
#[path = "crate_alias.rs"]
mod crate_alias;
#[path = "decimal.rs"]
#[cfg(any(feature = "rust_decimal1", feature = "bigdecimal04"))]
mod decimal;
#[path = "default.rs"]
mod default;
#[path = "deprecated.rs"]
mod deprecated;
#[path = "docs.rs"]
mod docs;
#[path = "either.rs"]
#[cfg(feature = "either1")]
mod either;
#[path = "enum_repr.rs"]
mod enum_repr;
#[path = "enums.rs"]
mod enums;
#[path = "enums_deny_unknown_fields.rs"]
mod enums_deny_unknown_fields;
#[path = "enums_flattened.rs"]
mod enums_flattened;
#[path = "enums_option_flattened.rs"]
mod enums_option_flattened;
#[path = "enums_ref_variants.rs"]
mod enums_ref_variants;
#[path = "enums_untagged_variant.rs"]
mod enums_untagged_variant;
#[path = "examples.rs"]
mod examples;
#[path = "extend.rs"]
mod extend;
#[path = "flatten.rs"]
mod flatten;
#[path = "from_into.rs"]
mod from_into;
#[path = "from_value.rs"]
mod from_value;
#[path = "garde.rs"]
mod garde;
#[path = "indexmap.rs"]
#[cfg(feature = "indexmap2")]
mod indexmap;
#[path = "inline_subschemas.rs"]
mod inline_subschemas;
#[path = "jiff.rs"]
#[cfg(feature = "jiff02")]
mod jiff;
#[path = "macros.rs"]
mod macros;
#[path = "map.rs"]
mod map;
#[path = "remote_derive.rs"]
mod remote_derive;
#[path = "same_name.rs"]
mod same_name;
#[path = "schema_name.rs"]
mod schema_name;
#[path = "schema_with.rs"]
mod schema_with;
#[path = "semver.rs"]
#[cfg(feature = "semver1")]
mod semver;
#[path = "settings.rs"]
mod settings;
#[path = "skip.rs"]
mod skip;
#[path = "smallvec.rs"]
#[cfg(feature = "smallvec1")]
mod smallvec;
#[path = "smol_str.rs"]
#[cfg(any(feature = "smol_str02", feature = "smol_str03"))]
mod smol_str;
#[path = "std_types.rs"]
mod std_types;
#[path = "structs.rs"]
mod structs;
#[path = "transform.rs"]
mod transform;
#[path = "transparent.rs"]
mod transparent;
#[path = "unset.rs"]
mod unset;
#[path = "url.rs"]
#[cfg(feature = "url2")]
mod url;
#[path = "uuid.rs"]
#[cfg(feature = "uuid1")]
mod uuid;
#[path = "validator.rs"]
mod validator;

mod prelude {
    pub(crate) use crate::test;
    pub(crate) use crate::test_helper::{arbitrary_values, arbitrary_values_except};
    pub(crate) use schemars::JsonSchema;
    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use serde_json::{json, Value};
}

#[path = "test_helper.rs"]
mod test_helper;

#[macro_export]
macro_rules! test_name {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let test_fn_name = type_name_of_val(f)
            .trim_end_matches("::f")
            .split("::")
            .last()
            .unwrap();
        format!("{}~{}", core::file!(), test_fn_name)
    }};
}

#[macro_export]
macro_rules! test {
    ($type:ty, $settings:expr) => {
        $crate::test_helper::TestHelper::<$type>::new($crate::test_name!(), $settings)
    };
    ($type:ty) => {
        test!($type, schemars::generate::SchemaSettings::default())
    };
    (value: $value:expr, $settings:expr) => {
        $crate::test_helper::TestHelper::new_for_value($crate::test_name!(), $settings, $value)
    };
    (value: $value:expr) => {
        test!(value: $value, schemars::generate::SchemaSettings::default())
    };
}
