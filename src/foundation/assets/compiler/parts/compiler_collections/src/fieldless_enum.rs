/// Macro to define a fieldless enum with a `VARIANTS` constant listing all variants in declaration order.
///
/// Wraps the enum definition and adds:
///
/// ```ignore
/// impl EnumName {
///     pub const VARIANTS: [EnumName; N] = [EnumName::A, EnumName::B, ...];
/// }
/// ```
#[macro_export]
macro_rules! fieldless_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $(= $discriminant:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $(= $discriminant)?
            ),*
        }

        impl $name {
            /// All variants in declaration order.
            $vis const VARIANTS: [$name; <[&str]>::len(&[$(stringify!($variant)),*])] = [
                $($name::$variant),*
            ];
        }
    };
}

pub use fieldless_enum;

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/fieldless_enum/tests.rs"]
#[cfg(test)]
mod tests;
