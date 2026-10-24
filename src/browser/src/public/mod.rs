//! Public route selection. Each page owns its state and DOM in its feature module.
use blog_interface::html;
mod appearance;
mod articles;
mod guestbook;
mod home;
mod journal;
mod love;
mod photos;
mod shell;
mod toolbox;

use crate::common::{load_typed, pathname, t};
use crate::models::Site;
use blog_interface::children::ToChildren;
use blog_interface::prelude::*;

#[component]
pub fn Public() -> impl IntoView {
    let site = load_typed::<Site>(Signal::derive(|| "/api/v1/site".into()));
    provide_context(site);
    match pathname().as_str() {
        "/" => home::Home().into_any(),
        "/sort" => {
            articles::Browse(articles::BrowseProps::builder().search(false).build()).into_any()
        }
        "/search" => {
            articles::Browse(articles::BrowseProps::builder().search(true).build()).into_any()
        }
        "/jotting" => journal::Journal().into_any(),
        "/menory" => journal::Journey().into_any(),
        "/message" => guestbook::Message().into_any(),
        "/travel" => photos::Photos().into_any(),
        "/favorite" => toolbox::Toolbox().into_any(),
        "/love" => love::Love().into_any(),
        "/letter" => love::Letter().into_any(),
        path if path.starts_with("/article/") => articles::Article(
            articles::ArticleProps::builder()
                .id(path[9..].parse::<i64>().unwrap_or(0))
                .build(),
        )
        .into_any(),
        _ => not_found().into_any(),
    }
}

fn not_found() -> impl IntoView {
    shell::Chrome(
        shell::ChromeProps::builder()
            .title(Signal::derive(|| t("页面不存在", "Page not found")))
            .children(ToChildren::to_children(move || {
                html::section().class("public-empty-state").child((
                    html::h2().child(t("页面不存在", "Page not found")),
                    html::a().child(t("返回首页", "Back to home")).href("/"),
                ))
            }))
            .build(),
    )
}
