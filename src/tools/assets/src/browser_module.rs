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
    generator
        .input_path(input)
        .out_name(name)
        .web(true)?
        .typescript(false)
        .omit_default_module_path(false);
    generator.generate(output)
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/browser_module.rs"]
mod tests;
