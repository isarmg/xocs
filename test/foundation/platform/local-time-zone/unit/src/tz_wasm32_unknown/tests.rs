
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn pass() {
        let tz = super::get_timezone_inner().unwrap();
        console_log!("tz={:?}", tz);
    }
