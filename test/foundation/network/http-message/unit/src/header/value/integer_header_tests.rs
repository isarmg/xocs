macro_rules! integer_header_tests {
    ($name:ident, $t:ident) => {
        #[test]
        fn $name() {
            let n: $t = 55;
            let val = HeaderValue::from(n);
            assert_eq!(val, &n.to_string());

            let n = ::std::$t::MAX;
            let val = HeaderValue::from(n);
            assert_eq!(val, &n.to_string());
        }
    };
}
