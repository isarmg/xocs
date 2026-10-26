//! Uses the same in-process browser module generator as the asset builder.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let input = arguments.next().expect("input WASM path is required");
    let output = arguments.next().expect("output directory is required");
    let mut generator = blog_wasm_browser_module::Bindgen::new();
    generator
        .input_path(input)
        .out_name("dom_contracts")
        .web(true)?
        .typescript(false);
    generator.generate(output)?;
    Ok(())
}
