//! Configured public media, safe background styles and text avatars.
use crate::common::{Data, safe_url};
use crate::models::Site;
use blog_interface::html;
use blog_interface::prelude::*;

fn css_string(value: &str) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for character in value.chars() {
        match character {
            '"' | '\\' => {
                result.push('\\');
                result.push(character);
            }
            character if character.is_control() => {
                write!(&mut result, "\\{:x} ", character as u32).expect("write to a string");
            }
            character => result.push(character),
        }
    }
    result.push('"');
    result
}

pub(super) fn background_style(url: Option<String>) -> String {
    url.map(|url| {
        format!(
            "background-image:linear-gradient(#0b2c3e4d,#0b2c3e73),url({})",
            css_string(&url)
        )
    })
    .unwrap_or_default()
}

pub(super) fn site_media_url(site: Data<Site>, field: fn(&Site) -> Option<&str>) -> Option<String> {
    site.value
        .get()
        .and_then(|site| field(&site).and_then(|url| safe_url(url.trim(), false)))
}

pub(super) fn site_name(site: Data<Site>) -> String {
    site.value
        .get()
        .and_then(|site| site.web_name.filter(|name| !name.trim().is_empty()))
        .unwrap_or_else(|| "blog".into())
}

#[component]
pub(super) fn Avatar(
    name: String,
    src: String,
    #[prop(default = "love-avatar")] class: &'static str,
    #[prop(default = String::new())] alt: String,
) -> impl IntoView {
    let image = safe_url(src.trim(), true);
    let fallback_label = (image.is_none() && !alt.is_empty()).then_some(alt.clone());
    View::new(
        html::span()
            .class(class)
            .attr("role", fallback_label.as_ref().map(|_| "img"))
            .aria_label(fallback_label)
            .child(match image {
                Some(url) => View::new(html::img().src(url).alt(alt)).into_any(),
                None => View::new(
                    html::span()
                        .child(
                            name.chars()
                                .find(|character| !character.is_whitespace())
                                .unwrap_or('•'),
                        )
                        .aria_hidden("true"),
                )
                .into_any(),
            }),
    )
}
