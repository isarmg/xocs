macro_rules! name_constant_tests {
    ($($id:ident),*) => {
        #[test]
        fn test_names_macro_consts() {
            #[allow(unused, deprecated)]
            use std::ascii::AsciiExt;
            $(
            assert_eq!($id.source.to_ascii_lowercase(), $id.source);
            )*
        }
    };
}
