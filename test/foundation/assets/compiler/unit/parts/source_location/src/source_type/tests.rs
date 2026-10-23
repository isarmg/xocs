
    use super::{FileExtension, SourceType};

    #[test]
    fn test_ts_from_path() {
        let ts = SourceType::from_path("foo.ts")
            .expect("foo.ts should be a valid TypeScript file path.");
        let mts = SourceType::from_path("foo.mts")
            .expect("foo.mts should be a valid TypeScript file path.");
        let cts = SourceType::from_path("foo.cts")
            .expect("foo.cts should be a valid TypeScript file path.");
        let tsx = SourceType::from_path("foo.tsx")
            .expect("foo.tsx should be a valid TypeScript file path.");

        for ty in &[ts, mts, cts, tsx] {
            assert!(ty.is_typescript());
            assert!(!ty.is_typescript_definition());
            assert!(!ty.is_javascript());
        }

        // Verify extension is captured
        assert_eq!(ts.extension(), Some(FileExtension::Ts));
        assert_eq!(mts.extension(), Some(FileExtension::Mts));
        assert_eq!(cts.extension(), Some(FileExtension::Cts));
        assert_eq!(tsx.extension(), Some(FileExtension::Tsx));

        // .ts and .tsx use Unambiguous (content-based detection)
        assert!(ts.is_unambiguous());
        assert!(tsx.is_unambiguous());
        // .mts is explicit ESM
        assert!(mts.is_module());
        // .cts is explicit CommonJS
        assert!(cts.is_commonjs());

        // Only explicit modules are strict
        assert!(!ts.is_strict()); // Unambiguous is not strict until resolved
        assert!(mts.is_strict());
        assert!(!cts.is_strict());
        assert!(!tsx.is_strict()); // Unambiguous is not strict until resolved

        assert!(!ts.is_jsx());
        assert!(!mts.is_jsx());
        assert!(!cts.is_jsx());
        assert!(tsx.is_jsx());
    }

    #[test]
    fn test_d_ts_from_path() {
        let dts = SourceType::from_path("foo.d.ts").unwrap();
        let dmts = SourceType::from_path("foo.d.mts").unwrap();
        let dcts = SourceType::from_path("foo.d.cts").unwrap();
        let arbitrary = SourceType::from_path("foo.d.ext.ts").unwrap();

        for ty in &[dts, dmts, dcts, arbitrary] {
            assert!(ty.is_typescript());
            assert!(ty.is_typescript_definition());
            assert!(!ty.is_javascript());
        }

        // .d.ts uses Unambiguous (content-based detection)
        assert!(dts.is_unambiguous());
        assert!(arbitrary.is_unambiguous());
        // .d.mts is explicit ESM
        assert!(dmts.is_module());
        // .d.cts is explicit CommonJS
        assert!(dcts.is_commonjs());

        // Only explicit modules are strict
        assert!(!dts.is_strict());
        assert!(dmts.is_strict());
        assert!(!dcts.is_strict());

        assert!(!dts.is_jsx());
        assert!(!dmts.is_jsx());
        assert!(!dcts.is_jsx());
    }

    #[test]
    fn test_js_from_path() {
        let js = SourceType::from_path("foo.js")
            .expect("foo.js should be a valid JavaScript file path.");
        let mjs = SourceType::from_path("foo.mjs")
            .expect("foo.mjs should be a valid JavaScript file path.");
        let cjs = SourceType::from_path("foo.cjs")
            .expect("foo.cjs should be a valid JavaScript file path.");
        let jsx = SourceType::from_path("foo.jsx")
            .expect("foo.jsx should be a valid JavaScript file path.");

        for ty in &[js, mjs, cjs, jsx] {
            assert!(ty.is_javascript(), "{ty:?}");
            assert!(!ty.is_typescript(), "{ty:?}");
        }

        // Verify extension is captured
        assert_eq!(mjs.extension(), Some(FileExtension::Mjs));
        assert_eq!(cjs.extension(), Some(FileExtension::Cjs));
        assert_eq!(js.extension(), Some(FileExtension::Js));
        assert_eq!(jsx.extension(), Some(FileExtension::Jsx));

        // .js and .jsx use Unambiguous (content-based detection)
        assert!(js.is_unambiguous());
        assert!(jsx.is_unambiguous());
        // .mjs is explicit ESM
        assert!(mjs.is_module());
        // .cjs is explicit CommonJS
        assert!(!cjs.is_module());

        // Only explicit modules are strict
        assert!(!js.is_strict()); // Unambiguous is not strict until resolved
        assert!(mjs.is_strict());
        assert!(!cjs.is_strict());
        assert!(!jsx.is_strict()); // Unambiguous is not strict until resolved

        assert!(js.is_javascript());
        assert!(mjs.is_javascript());
        assert!(cjs.is_javascript());
        assert!(jsx.is_jsx());
    }
