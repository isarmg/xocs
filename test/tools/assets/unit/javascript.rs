use super::*;
use anyhow::Result;
#[test]
fn strings_comments_regex_are_not_module_edges() -> Result<()> {
    let text = r#"// import x from 'fake';
const s="import x from 'fake'"; const r=/import\("fake"\)/;
import {x} from "./real.js"; export { x as y } from './other.js';
const asset=new URL('./a.wasm',import.meta.url);"#;
    let result = references(text)?;
    assert_eq!(result.imports.len(), 2);
    assert_eq!(result.assets.len(), 1);
    assert_eq!(result.imports[0].url, "./real.js");
    assert!(references("import(window.name)").is_err());
    Ok(())
}
#[test]
fn syntax_semantics_and_module_loading_contract_remain_strict() -> Result<()> {
    for source in [
        "export const = ;",
        "const x = 1; const x = 2;",
        "import(window.name)",
        "import('./a.js', {with: {type: 'json'}})",
        "const item = require('./a.js');",
    ] {
        assert!(
            references(source).is_err(),
            "Accepted invalid module: {source}"
        );
    }
    let source = "export { x } from './static.js'; const item = import('./dynamic.js');";
    let refs = references(source)?;
    assert_eq!(refs.imports.len(), 1);
    assert_eq!(refs.imports[0].url, "./static.js");
    assert_eq!(refs.dynamic.len(), 1);
    assert_eq!(refs.dynamic[0].0, "./dynamic.js");
    let (start, end) = range(refs.imports[0].source);
    assert_eq!(&source[start..end], "'./static.js'");
    Ok(())
}
#[test]
fn global_asset_references_respect_lexical_bindings() -> Result<()> {
    let source = r#"
            export function local(URL, require) {
                require('./not-a-module');
                return new URL('./not-an-asset', import.meta.url);
            }
            export function hoisted() {
                const value = new URL('./also-not-an-asset', import.meta.url);
                function URL() {}
                return value;
            }
            export const real = new URL('./real.wasm', import.meta.url);
        "#;
    let refs = references(source)?;
    assert_eq!(refs.assets.len(), 1);
    assert_eq!(refs.assets[0].0, "./real.wasm");
    assert!(references("require('./unsupported.js')").is_err());
    Ok(())
}

#[test]
fn static_template_urls_use_cooked_values_and_whole_literal_spans() -> Result<()> {
    let source =
        r"export const asset = new URL(`./a\u002ewasm`, import.meta.url); import(`./module.js`);";
    let refs = references(source)?;
    assert_eq!(refs.assets[0].0, "./a.wasm");
    let (start, end) = range(refs.assets[0].1);
    assert_eq!(&source[start..end], r"`./a\u002ewasm`");
    assert_eq!(refs.dynamic[0].0, "./module.js");
    assert!(references("import(`./${name}.js`)").is_err());
    Ok(())
}

#[test]
fn export_attributes_cannot_bypass_the_module_contract() {
    for source in [
        "export { default } from './data.json' with { type: 'json' };",
        "export * from './data.json' with { type: 'json' };",
    ] {
        assert!(references(source).is_err(), "Accepted {source}");
    }
}

#[test]
fn compression_preserves_string_and_regular_expression_literals() -> Result<()> {
    let code = minify("export const s = 'a  b'; export const r = /a  b/; // comment\n")?;
    let result = String::from_utf8(code)?;
    assert!(result.contains("a  b"));
    assert!(!result.contains("// comment"));
    references(&result)?;
    Ok(())
}

#[test]
fn compiler_failures_keep_structured_diagnostics_and_feature_reasons() {
    use std::error::Error;
    let error = references("export const = ;").unwrap_err();
    assert!(matches!(
        error,
        JavaScriptError::Diagnostics {
            stage: JavaScriptStage::Source,
            ..
        }
    ));
    assert!(error.source().unwrap().is::<OxcDiagnostic>());
    let error = references("import(window.name); require('./legacy.js');").unwrap_err();
    let JavaScriptError::Unsupported {
        primary,
        additional,
    } = error
    else {
        panic!("expected unsupported module features");
    };
    assert!(matches!(
        primary,
        UnsupportedModuleFeature::ComputedDynamicImport
    ));
    assert!(matches!(
        additional.as_slice(),
        [UnsupportedModuleFeature::CommonJs]
    ));
}
