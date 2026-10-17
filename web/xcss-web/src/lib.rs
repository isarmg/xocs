use xcss_web::component::{component_props_builder, component_view};
mod admin;
mod browser;
mod comments;
mod common;
mod contracts;
mod dates;
mod editor;
mod markdown;
mod models;
mod public;
mod select;
use xcss_web::prelude::*;
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    browser::install_document_policies();
    xcss_web::mount("root", || {
        if common::pathname().starts_with("/admin") {
            View::new({
                #[allow(unreachable_code)]
                #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &admin::Admin,
                    {
                        let mut props = component_props_builder(&admin::Admin).build();
                        props
                    },
                )
            })
            .into_any()
        } else {
            View::new({
                #[allow(unreachable_code)]
                #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &public::Public,
                    {
                        let mut props = component_props_builder(&public::Public).build();
                        props
                    },
                )
            })
            .into_any()
        }
    });
}
