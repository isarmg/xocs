//! Browser-only WASM module generation, integrated into the asset compiler.
//! Original source and legal records: compliance/provenance/blog-assets.json.
use std::path::{Path, PathBuf};
use wasm_bindgen_cli_support::Bindgen;

/// Failures at the external WASM binding tool boundary.
#[derive(Debug, thiserror::Error)]
pub enum BrowserModuleError {
    #[error("invalid browser module name")]
    InvalidName,
    #[error("generate browser module from {input:?} in {output:?}")]
    Generate {
        input: PathBuf,
        output: PathBuf,
        #[source]
        source: anyhow::Error,
    },
}

pub(crate) fn generate(input: &Path, output: &Path, name: &str) -> Result<(), BrowserModuleError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(BrowserModuleError::InvalidName);
    }
    let mut generator = Bindgen::new();
    generator
        .input_path(input)
        .out_name(name)
        .web(true)
        .expect("invariant: a new binding generator has no conflicting output mode")
        .typescript(false)
        .omit_default_module_path(false);
    generator
        .generate(output)
        .map_err(|source| BrowserModuleError::Generate {
            input: input.to_path_buf(),
            output: output.to_path_buf(),
            source,
        })
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/browser_module.rs"]
mod tests;
