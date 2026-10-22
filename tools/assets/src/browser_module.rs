//! Browser-only WASM module generation, integrated into the asset compiler.
//! Original source and legal records: compliance/provenance/blog-assets.json.
use anyhow::{Result, ensure};
use std::path::Path;
use wasm_bindgen_cli_support::Bindgen;

pub(crate) fn generate(input: &Path, output: &Path, name: &str) -> Result<()> {
    ensure!(
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
        "Invalid browser module name"
    );
    let mut generator = Bindgen::new();
    #[allow(deprecated)]
    generator
        .input_path(input)
        .out_name(name)
        .web(true)?
        .typescript(false)
        .omit_default_module_path(false)
        .reference_types(false);
    generator.generate(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_browser_initialization_and_default_url_without_typescript() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let input = temporary.path().join("empty.wasm");
        let output = temporary.path().join("output");
        std::fs::write(&input, b"\0asm\x01\0\0\0")?;
        generate(&input, &output, "browser")?;
        let javascript = std::fs::read_to_string(output.join("browser.js"))?;
        assert!(
            javascript.contains("export default") || javascript.contains("__wbg_init as default")
        );
        assert!(javascript.contains("new URL"));
        assert!(!javascript.contains("import * as wasm from"));
        assert!(!output.join("browser.d.ts").exists());
        Ok(())
    }

    #[test]
    fn invalid_module_names_do_not_create_outputs() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let output = temporary.path().join("output");
        for name in ["", "../outside", "nested/name", "name.js"] {
            assert!(generate(&temporary.path().join("unused.wasm"), &output, name).is_err());
            assert!(!output.exists());
        }
        Ok(())
    }
}
