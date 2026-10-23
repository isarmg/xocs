
    use super::*;
    use crate::transforms::parse_wat;
    use walrus::ir::{Instr, InstrSeq, Visitor};
    use walrus::FunctionKind;

    const HOOKS_WAT: &str = r#"
        (func $__jspi_enter (result i64) i64.const 0)
        (func $__jspi_exit (param i64 i32))
        (func $__jspi_suspend (result i64) i64.const 0)
        (func $__jspi_resume (param i64 i32))
        (export "__jspi_enter" (func $__jspi_enter))
        (export "__jspi_exit" (func $__jspi_exit))
        (export "__jspi_suspend" (func $__jspi_suspend))
        (export "__jspi_resume" (func $__jspi_resume))
        (func $__wbindgen_jspi_set_rejected (param i32))
    "#;

    fn module(catch: bool, legacy: bool) -> (Module, ExportId, FunctionId) {
        let import = if catch {
            r#"(import "env" "sleep" (func $sleep (param i32) (result externref)))"#
        } else {
            r#"(import "env" "sleep" (func $sleep (param i32) (result i32)))"#
        };
        let body = if legacy {
            "try i32.const 1 drop catch_all end"
        } else {
            ""
        };
        let wat = format!(
            r#"(module
                {import}
                (memory 1)
                {HOOKS_WAT}
                (func $work (param i32) (result i32)
                    {body}
                    local.get 0
                    call $sleep
                    drop
                    local.get 0)
                (export "work" (func $work)))"#
        );
        let module = parse_wat(&wat);
        let export = module
            .exports
            .iter()
            .find(|e| e.name == "work")
            .unwrap()
            .id();
        let import = module.funcs.by_name("sleep").unwrap();
        (module, export, import)
    }

    fn run(catch: bool, legacy: bool) -> Module {
        let (mut module, export, import) = module(catch, legacy);
        let aux = WasmBindgenAux {
            jspi_set_rejected: module.funcs.by_name("__wbindgen_jspi_set_rejected"),
            ..Default::default()
        };
        let mut wit = NonstandardWitSection::default();
        super::run(
            &mut module,
            &aux,
            &mut wit,
            vec![export],
            vec![(import, catch)],
            legacy,
        )
        .unwrap();
        let features =
            wasmparser::WasmFeatures::default() | wasmparser::WasmFeatures::LEGACY_EXCEPTIONS;
        wasmparser::Validator::new_with_features(features)
            .validate_all(&module.emit_wasm())
            .expect("transformed module should validate");
        module
    }

    /// The hook calls on every path of `func`, in order of appearance.
    fn hook_calls(module: &Module, func: FunctionId) -> Vec<String> {
        struct Scan<'a> {
            module: &'a Module,
            calls: Vec<String>,
            legacy_try: usize,
            try_table: usize,
        }
        impl<'instr> Visitor<'instr> for Scan<'_> {
            fn start_instr_seq(&mut self, seq: &'instr InstrSeq) {
                for (instr, _) in &seq.instrs {
                    match instr {
                        Instr::Call(c) => {
                            if let Some(name) = &self.module.funcs.get(c.func).name {
                                if name.starts_with("__jspi_") {
                                    self.calls.push(name.clone());
                                }
                            }
                        }
                        Instr::Try(_) => self.legacy_try += 1,
                        Instr::TryTable(_) => self.try_table += 1,
                        _ => {}
                    }
                }
            }
        }
        let FunctionKind::Local(local) = &module.funcs.get(func).kind else {
            panic!("wrapper should be local");
        };
        let mut scan = Scan {
            module,
            calls: Vec::new(),
            legacy_try: 0,
            try_table: 0,
        };
        ir::dfs_in_order(&mut scan, local, local.entry_block());
        scan.calls.push(format!(
            "try={} try_table={}",
            scan.legacy_try, scan.try_table
        ));
        scan.calls
    }

    fn wrapper_of_export(module: &Module, name: &str) -> FunctionId {
        match module.exports.iter().find(|e| e.name == name).unwrap().item {
            ExportItem::Function(f) => f,
            _ => unreachable!(),
        }
    }

    fn check(catch: bool, legacy: bool) {
        let module = run(catch, legacy);
        let eh = if legacy {
            "try=1 try_table=0"
        } else {
            "try=0 try_table=1"
        };

        let export_wrapper = wrapper_of_export(&module, "work");
        assert_ne!(export_wrapper, module.funcs.by_name("work").unwrap());
        assert_eq!(
            hook_calls(&module, export_wrapper),
            ["__jspi_enter", "__jspi_exit", "__jspi_exit", eh]
        );

        // `work` now calls the suspending wrapper, not the import.
        let import = module.funcs.by_name("sleep").unwrap();
        let work = module.funcs.by_name("work").unwrap();
        let FunctionKind::Local(local) = &module.funcs.get(work).kind else {
            unreachable!()
        };
        let mut callee = None;
        ir::dfs_in_order(&mut CalleeScan(&mut callee), local, local.entry_block());
        let import_wrapper = callee.expect("work should call something");
        assert_ne!(import_wrapper, import);
        let resumes = if catch { 3 } else { 2 };
        let mut expected = vec!["__jspi_suspend".to_string()];
        expected.extend(std::iter::repeat_n("__jspi_resume".to_string(), resumes));
        expected.push(eh.to_string());
        assert_eq!(hook_calls(&module, import_wrapper), expected);
    }

    struct CalleeScan<'a>(&'a mut Option<FunctionId>);
    impl<'instr> Visitor<'instr> for CalleeScan<'_> {
        fn visit_call(&mut self, call: &ir::Call) {
            *self.0 = Some(call.func);
        }
    }

    #[test]
    fn modern() {
        check(false, false);
    }

    #[test]
    fn modern_catch() {
        check(true, false);
    }

    #[test]
    fn legacy() {
        check(false, true);
    }

    #[test]
    fn legacy_catch() {
        check(true, true);
    }

    #[test]
    fn missing_hooks() {
        let mut module = parse_wat(r#"(module (func $work) (export "work" (func $work)))"#);
        let export = module.exports.iter().next().unwrap().id();
        let err = super::run(
            &mut module,
            &WasmBindgenAux::default(),
            &mut NonstandardWitSection::default(),
            vec![export],
            vec![],
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("-sJSPI_HOOKS"), "{err}");
    }
