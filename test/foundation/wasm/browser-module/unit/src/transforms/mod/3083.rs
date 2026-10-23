#[cfg(test)]
pub(crate) fn parse_wat(wat: &str) -> walrus::Module {
    let wasm = wat::parse_str(wat).unwrap();
    walrus::ModuleConfig::new()
        .generate_producers_section(false)
        .parse(&wasm)
        .unwrap()
}
