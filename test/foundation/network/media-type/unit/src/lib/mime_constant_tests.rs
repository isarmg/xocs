macro_rules! mime_constant_tests {
    ($($id:ident, $($piece:expr),*;)*) => {
        #[test]
        fn test_mimes_macro_consts() {
            let _ = [
            $(
            mime_constant_test! {
                $id, $($piece),*
            }
            ),*
            ].iter().enumerate().map(|(pos, &atom)| {
                assert_eq!(pos + 1, atom as usize, "atom {} in position {}", atom, pos + 1);
            }).collect::<Vec<()>>();
        }
    };
}
