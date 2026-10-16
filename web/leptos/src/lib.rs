mod admin;
mod browser;
mod comments;
mod common;
mod contracts;
mod dates;
mod editor;
mod public;
mod select;
use leptos::prelude::*;
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    let _policies = browser::install_document_policies();
    xcss_web::mount("root", || {
        if common::pathname().starts_with("/admin") {
            view! {<admin::Admin/>}.into_any()
        } else {
            view! {<public::Public/>}.into_any()
        }
    });
}
