
    use std::{path::Path, str::FromStr};

    use super::FileExtension;

    #[test]
    fn test_is_ts_declaration() {
        let cases = vec![
            ("index.d.ts", true),
            ("index.ts", false),
            ("d.ts", false),
            (".d.ts", false),
            ("index.d.mts", true),
            ("index.mts", false),
            ("d.mts", false),
            (".d.mts", false),
            ("index.d.cts", true),
            ("index.cts", false),
            ("d.cts", false),
            (".d.cts", false),
            ("index.d.js", false),
            ("index.js", false),
            ("index.d.jsx", false),
            ("index.jsx", false),
            ("index.d.css.ts", true),
            ("index.d.css.mts", false),
        ];

        for (file_name, expected) in cases {
            let ext = Path::new(file_name)
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .expect("File name should have an extension.");
            let ext = FileExtension::from_str(ext).expect("Extension should be valid.");

            assert_eq!(
                ext.is_ts_declaration(file_name),
                expected,
                "Expected is_ts_declaration to be {} for file name {} but got {}",
                expected,
                file_name,
                !expected
            );
        }
    }
