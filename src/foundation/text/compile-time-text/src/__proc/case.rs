/// Converts a string literal to lowercase or uppercase.
///
/// ```
/// assert_eq!(const_str::convert_case!(lower, "Lower Case"), "lower case");
/// assert_eq!(const_str::convert_case!(upper, "Upper Case"), "UPPER CASE");
/// ```
#[macro_export]
macro_rules! convert_case {
    (lower, $s: literal) => {
        $crate::__proc::to_lowercase!($s)
    };
    (upper, $s: literal) => {
        $crate::__proc::to_uppercase!($s)
    };
}
