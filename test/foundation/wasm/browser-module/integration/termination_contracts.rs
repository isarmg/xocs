use blog_wasm_browser_module::{Bindgen, Output};
use walrus::{ir::Value, ConstExpr, Module, RefType, ValType};

fn generate(mut module: Module) -> anyhow::Result<Output> {
    let mut bindgen = Bindgen::new();
    bindgen
        .input_bytes("termination_contract", module.emit_wasm())
        .force_enable_abort_handler(true)
        .web(true)?;
    bindgen.generate_output()
}

fn with_memory() -> Module {
    let mut module = Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);
    module.exports.add("memory", memory);
    let table = module.tables.add_local(false, 1, None, RefType::FUNCREF);
    module.exports.add("contract_table", table);
    module
}

#[test]
fn a_module_without_the_optional_termination_symbol_is_accepted() {
    assert!(generate(with_memory()).is_ok());
}

#[test]
fn a_local_i32_termination_address_is_accepted() {
    let mut module = with_memory();
    let address = module.globals.add_local(
        ValType::I32,
        false,
        false,
        ConstExpr::Value(Value::I32(128)),
    );
    module.exports.add("__instance_terminated", address);
    if let Err(error) = generate(module) {
        panic!("a valid termination symbol must be accepted: {error:#}");
    }
}

#[test]
fn an_i64_termination_address_is_rejected() {
    let mut module = with_memory();
    let address = module.globals.add_local(
        ValType::I64,
        false,
        false,
        ConstExpr::Value(Value::I64(128)),
    );
    module.exports.add("__instance_terminated", address);
    assert_invalid_termination_symbol(module);
}

#[test]
fn an_imported_termination_address_is_rejected() {
    let mut module = with_memory();
    let (address, _) =
        module.add_import_global("env", "termination_address", ValType::I32, false, false);
    module.exports.add("__instance_terminated", address);
    assert_invalid_termination_symbol(module);
}

#[test]
fn a_non_global_termination_export_is_rejected() {
    let mut module = with_memory();
    let memory = module
        .memories
        .iter()
        .next()
        .expect("a test memory exists")
        .id();
    module.exports.add("__instance_terminated", memory);
    assert_invalid_termination_symbol(module);
}

fn assert_invalid_termination_symbol(module: Module) {
    match generate(module) {
        Ok(_) => panic!("an invalid existing termination symbol must be rejected"),
        Err(error) => {
            let chain = format!("{error:#}");
            assert!(
                chain.contains("failed to generate catch wrappers"),
                "{chain}"
            );
            assert!(
                chain.contains("__instance_terminated must be a local i32 constant global"),
                "{chain}"
            );
        }
    }
}
