macro_rules! standard_header_tests {
    ($(($konst:ident, $name_bytes:literal));+ $(;)?) => {
        #[cfg(test)]
        const TEST_HEADERS: &'static [(StandardHeader, &'static [u8])] = &[
            $(
            (StandardHeader::$konst, $name_bytes),
            )+
        ];

        #[test]
        fn test_parse_standard_headers() {
            for &(std, name_bytes) in TEST_HEADERS {
                // Test lower case
                assert_eq!(HeaderName::from_bytes(name_bytes).unwrap(), HeaderName::from(std));

                // Test upper case
                let upper = std::str::from_utf8(name_bytes).expect("byte string constants are all utf-8").to_uppercase();
                assert_eq!(HeaderName::from_bytes(upper.as_bytes()).unwrap(), HeaderName::from(std));
            }
        }

        #[test]
        fn test_standard_headers_into_bytes() {
            for &(std, name_bytes) in TEST_HEADERS {
                let name = std::str::from_utf8(name_bytes).unwrap();
                let std = HeaderName::from(std);
                // Test lower case
                let bytes: Bytes =
                    HeaderName::from_bytes(name_bytes).unwrap().inner.into();
                assert_eq!(bytes, name);
                assert_eq!(HeaderName::from_bytes(name_bytes).unwrap(), std);

                // Test upper case
                let upper = name.to_uppercase();
                let bytes: Bytes =
                    HeaderName::from_bytes(upper.as_bytes()).unwrap().inner.into();
                assert_eq!(bytes, name_bytes);
                assert_eq!(HeaderName::from_bytes(upper.as_bytes()).unwrap(),
                           std);
            }

        }
    };
}
