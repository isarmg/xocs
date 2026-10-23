use alloc::format;
use alloc::string::String;

// Expand the same declarations as the encoder and decoder. The compiler strips
// comments before this macro runs; helpers and Rustdoc are outside the snapshot.
macro_rules! schema_signature {
    ($($kind:ident $name:ident $(<$lifetime:lifetime>)? { $($body:tt)* })*) => {
        fn schema_signature() -> String {
            let mut output = String::from(super::SCHEMA_VERSION);
            output.push('\n');
            $(
                output.push_str(stringify!($kind));
                output.push(' ');
                output.push_str(stringify!($name));
                $(
                    output.push('<');
                    output.push_str(stringify!($lifetime));
                    output.push('>');
                )?
                output.push_str(" { ");
                output.extend(stringify!($($body)*).chars().filter(|c| !c.is_whitespace()));
                output.push_str(" }\n");
            )*
            for (name, value) in super::tys::DESCRIPTORS {
                output.push_str(&format!("{name}={value}\n"));
            }
            output
        }
    };
}

crate::shared_api!(schema_signature);

#[test]
fn schema_matches_approved_wire_contract() {
    assert_eq!(
        schema_signature(),
        include_str!("../../../../../../src/foundation/wasm/browser-bridge-schema/src/schema.approved"),
        "Review the wire contract and bump SCHEMA_VERSION for protocol changes"
    );
}
