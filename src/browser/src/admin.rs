mod articles;
mod dashboard;
mod family;
mod home;
mod letter;
mod links;
mod moderation;
mod site;
mod taxonomy;

use self::articles::Articles;
use self::dashboard::Dashboard;
use self::family::FamilySettings;
use self::home::HomeSections;
use self::letter::LetterSettings;
use self::links::Links;
use self::moderation::{Moderation, ModerationKind};
use self::site::SiteSettings;
use self::taxonomy::Taxonomy;
use crate::common::{hash_signal, href, navigate, t};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::components::{AccountPage, AdminShell, HeaderActions, HeaderNavigation};
use blog_interface::prelude::*;

/// One canonical identity for each page that appears in the admin navigation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    Articles,
    Categories,
    Labels,
    Notes,
    Letter,
    Comments,
    Messages,
    Files,
    Links,
    Family,
    Site,
    Home,
}

impl Page {
    fn hash(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Articles => "articles",
            Self::Categories => "categories",
            Self::Labels => "labels",
            Self::Notes => "notes",
            Self::Letter => "letter",
            Self::Comments => "comments",
            Self::Messages => "tree-hole",
            Self::Files => "resources",
            Self::Links => "links",
            Self::Family => "family",
            Self::Site => "site",
            Self::Home => "home",
        }
    }

    fn label(self) -> String {
        match self {
            Self::Dashboard => t("总览", "Overview"),
            Self::Articles => t("文章管理", "Articles"),
            Self::Categories => t("分类管理", "Categories"),
            Self::Labels => t("标签管理", "Tags"),
            Self::Notes => t("随笔管理", "Notes"),
            Self::Letter => t("信笺管理", "Letter"),
            Self::Comments => t("评论管理", "Comments"),
            Self::Messages => t("留言板管理", "Message board"),
            Self::Files => t("文件管理", "Files"),
            Self::Links => t("内容资源", "Content resources"),
            Self::Family => t("恋爱笔记", "Love journal"),
            Self::Site => t("网站设置", "Site settings"),
            Self::Home => t("首页栏目", "Home sections"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Group {
    Overview,
    Content,
    Community,
    Resources,
    Settings,
}

impl Group {
    const ALL: [Self; 5] = [
        Self::Overview,
        Self::Content,
        Self::Community,
        Self::Resources,
        Self::Settings,
    ];

    fn pages(self) -> &'static [Page] {
        match self {
            Self::Overview => &[Page::Dashboard],
            Self::Content => &[
                Page::Articles,
                Page::Categories,
                Page::Labels,
                Page::Notes,
                Page::Letter,
            ],
            Self::Community => &[Page::Comments, Page::Messages],
            Self::Resources => &[Page::Files, Page::Links, Page::Family],
            Self::Settings => &[Page::Site, Page::Home],
        }
    }

    fn label(self) -> String {
        match self {
            Self::Overview => t("总览", "Overview"),
            Self::Content => t("内容", "Content"),
            Self::Community => t("社区", "Community"),
            Self::Resources => t("资源", "Resources"),
            Self::Settings => t("设置", "Settings"),
        }
    }

    fn title(self) -> String {
        match self {
            Self::Overview => t("总览", "Overview"),
            Self::Content => t("内容管理", "Content management"),
            Self::Community => t("社区管理", "Community management"),
            Self::Resources => t("资源管理", "Resource management"),
            Self::Settings => t("网站设置", "Site settings"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Page(Page),
    Account,
    NewArticle,
    EditArticle(i64),
}

impl Route {
    fn parse(hash: &str) -> Self {
        if hash == "account" {
            Self::Account
        } else if hash == "new" {
            Self::NewArticle
        } else if let Some(id) = hash.strip_prefix("edit/") {
            Self::EditArticle(id.parse().unwrap_or(0))
        } else {
            Self::Page(
                Group::ALL
                    .into_iter()
                    .flat_map(|group| group.pages().iter().copied())
                    .find(|page| page.hash() == hash)
                    .unwrap_or(Page::Articles),
            )
        }
    }

    fn active_page(self) -> Option<Page> {
        match self {
            Self::Page(page) => Some(page),
            Self::Account => None,
            Self::NewArticle | Self::EditArticle(_) => Some(Page::Articles),
        }
    }

    fn group(self) -> Option<Group> {
        self.active_page().and_then(|page| {
            Group::ALL
                .into_iter()
                .find(|group| group.pages().contains(&page))
        })
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    View::new(
        blog_interface::html::div()
            .class("blog-admin-app")
            .child(AdminShell(
                component_props_builder(&AdminShell)
                    .product("blog")
                    .landing("/admin#articles")
                    .children(ToChildren::to_children(Workspace))
                    .build(),
            )),
    )
}

#[component]
fn Workspace() -> impl IntoView {
    let hash = hash_signal();
    let route = Memo::new(move |_| Route::parse(&hash.get()));
    View::new(
        blog_interface::html::div()
            .class("blog-admin-workspace blog-content-stack blog-stack")
            .child((
                HeaderNavigation(
                    component_props_builder(&HeaderNavigation)
                        .label(t("后台导航", "Administration navigation"))
                        .children(ToChildren::to_children(move || {
                            Group::ALL
                                .into_iter()
                                .map(move |group| {
                                    View::new(
                                        blog_interface::html::button()
                                            .class("blog-button")
                                            .child(group.label())
                                            .aria_pressed(move || {
                                                (route.get().group() == Some(group)).to_string()
                                            })
                                            .on(blog_interface::ev::click, move |_| {
                                                navigate(group.pages()[0].hash())
                                            }),
                                    )
                                })
                                .collect_view()
                        }))
                        .build(),
                ),
                HeaderActions(
                    component_props_builder(&HeaderActions)
                        .children(ToChildren::to_children(move || {
                            blog_interface::html::a()
                                .class("blog-site-link blog-button")
                                .href(href("/"))
                                .aria_label(t("返回网站", "Back to site"))
                                .title(t("返回网站", "Back to site"))
                                .child(
                                    blog_interface::svg::svg()
                                        .attr("aria-hidden", "true")
                                        .attr("width", "1em")
                                        .attr("height", "1em")
                                        .attr("viewBox", "0 0 24 24")
                                        .attr("fill", "none")
                                        .attr("stroke", "currentColor")
                                        .attr("stroke-width", "1.7")
                                        .child(
                                            blog_interface::svg::path()
                                                .attr("d", "m3 10 9-7 9 7M5 9v11h5v-7h4v7h5V9"),
                                        ),
                                )
                        }))
                        .build(),
                ),
                move || {
                    route
                        .get()
                        .group()
                        .filter(|group| group.pages().len() > 1)
                        .map(|group| {
                            View::new(
                                blog_interface::html::nav()
                                    .class("blog-secondary-navigation blog-flow")
                                    .aria_label(group.title())
                                    .child(
                                        group
                                            .pages()
                                            .iter()
                                            .copied()
                                            .map(move |page| {
                                                View::new(
                                                    blog_interface::html::button()
                                                        .class("blog-button")
                                                        .child(page.label())
                                                        .aria_pressed(move || {
                                                            (route.get().active_page()
                                                                == Some(page))
                                                            .to_string()
                                                        })
                                                        .on(blog_interface::ev::click, move |_| {
                                                            navigate(page.hash())
                                                        }),
                                                )
                                            })
                                            .collect_view(),
                                    ),
                            )
                        })
                },
                move || render_page(route.get()),
            )),
    )
}

fn render_page(route: Route) -> AnyView {
    match route {
        Route::Account => View::new(AccountPage()).into_any(),
        Route::NewArticle => View::new(crate::editor::ArticleEditor(
            component_props_builder(&crate::editor::ArticleEditor).build(),
        ))
        .into_any(),
        Route::EditArticle(id) => View::new(crate::editor::ArticleEditor(
            component_props_builder(&crate::editor::ArticleEditor)
                .id(id)
                .build(),
        ))
        .into_any(),
        Route::Page(page) => match page {
            Page::Dashboard => View::new(Dashboard()).into_any(),
            Page::Articles => View::new(Articles()).into_any(),
            Page::Categories => View::new(Taxonomy(
                component_props_builder(&Taxonomy).labels(false).build(),
            ))
            .into_any(),
            Page::Labels => View::new(Taxonomy(
                component_props_builder(&Taxonomy).labels(true).build(),
            ))
            .into_any(),
            Page::Notes => View::new(crate::editor::Notes()).into_any(),
            Page::Letter => View::new(LetterSettings()).into_any(),
            Page::Comments => View::new(Moderation(
                component_props_builder(&Moderation)
                    .kind(ModerationKind::Comments)
                    .build(),
            ))
            .into_any(),
            Page::Messages => View::new(Moderation(
                component_props_builder(&Moderation)
                    .kind(ModerationKind::Messages)
                    .build(),
            ))
            .into_any(),
            Page::Files => View::new(Moderation(
                component_props_builder(&Moderation)
                    .kind(ModerationKind::Files)
                    .build(),
            ))
            .into_any(),
            Page::Links => View::new(Links()).into_any(),
            Page::Family => View::new(FamilySettings()).into_any(),
            Page::Site => View::new(SiteSettings()).into_any(),
            Page::Home => View::new(HomeSections()).into_any(),
        },
    }
}

#[cfg(test)]
#[path = "../../../test/browser/unit/admin/routes.rs"]
mod tests;
