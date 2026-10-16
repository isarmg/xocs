use wasm_bindgen::prelude::*;
#[wasm_bindgen(module = "/src/browser.js")]
extern "C" {
    #[wasm_bindgen(js_name=installDocumentPolicies)]
    pub fn install_document_policies() -> js_sys::Function;
    #[wasm_bindgen(js_name=markdownHtml)]
    pub fn markdown_html(s: &str) -> String;
    #[wasm_bindgen(js_name=readingMarkup)]
    pub fn reading_markup(s: &str, copy: &str, image: &str, section: &str) -> String;
    #[wasm_bindgen(js_name=observeSliding)]
    pub fn observe_sliding(element: &web_sys::Element) -> js_sys::Function;
}
