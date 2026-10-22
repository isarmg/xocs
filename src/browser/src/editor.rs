use crate::common::Action;
use crate::common::*;
use crate::models::{Article, ArticleDraft, Category, Label, News, Page};
use blog_interface::component::{component_props_builder, component_view};
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn ArticleEditor(#[prop(default = 0)] id: i64) -> impl IntoView {
    let categories = load_typed::<Vec<Category>>(Signal::derive(|| "/api/v1/categories".into()));
    let labels = load_typed::<Vec<Label>>(Signal::derive(|| "/api/v1/content/labels".into()));
    let article = load_typed::<Article>(Signal::derive(move || {
        if id > 0 {
            format!("/api/v1/content/articles/{id}")
        } else {
            String::new()
        }
    }));
    let form = RwSignal::new(ArticleDraft::default());
    let existing = RwSignal::new(false);
    let ready = RwSignal::new(id == 0);
    let preview = RwSignal::new(false);
    let save = Action::new();
    let upload = Action::new();
    let validation = RwSignal::new(String::new());
    let creation = StoredValue::new(web_sys::window().unwrap().crypto().unwrap().random_uuid());
    let created_id = StoredValue::new(None::<i64>);
    Effect::new(move |_| {
        if let Some(v) = article.value.get() {
            form.set(ArticleDraft::from(&v));
            existing.set(v.password_required);
            ready.set(true);
        }
    });
    let upload_file = move |file: blog_interface::thread_bound::SendWrapper<web_sys::File>,
                            cover: bool| {
        if !ready.get_untracked() || save.busy.get_untracked() {
            return;
        }
        let name = file.name().replace(['[', ']'], "");
        upload.upload(file, move |v| {
            let path = text(&v, "path");
            form.update(|f| {
                if cover {
                    f.article_cover = Some(path)
                } else {
                    f.article_content = format!("{}\n\n![{name}]({path})\n", f.article_content)
                }
            });
            notify(t("图片已上传", "Image uploaded"));
        });
    };
    View::new(
    blog_interface::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if id > 0 {
                    t("编辑文章", "Edit article") } else { t("新增文章",
                    "Add article") } }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("支持 Markdown 正文，发布后可从网站访问。",
                    "Write in Markdown. Published articles are available on the site.")
                    }) })), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { categories.error })
                    .fallback(#[allow(unused_braces)] { t("分类加载失败",
                    "Unable to load categories") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | categories.reload())) }).build();
                    props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { labels.error })
                    .fallback(#[allow(unused_braces)] { t("标签加载失败",
                    "Unable to load tags") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | labels.reload())) }).build();
                    props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { article.error })
                    .fallback(#[allow(unused_braces)] { t("文章加载失败",
                    "Unable to load article") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | article.reload())) }).build();
                    props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error })
                    .fallback(#[allow(unused_braces)] {
                    t("尚未确认文章是否已创建，请保留原内容重试。",
                    "Article creation is not yet confirmed. Retry with the original content unchanged.")
                    }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { upload.error }).build(); props
                    }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! validation.get().is_empty()).then(| | View::new(
    blog_interface::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ validation.get() }) },
        )
        .role("alert"),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Show, { let mut props =
                    component_props_builder(& Show)
                    .when(#[allow(unused_braces)] { move | | ready.get() })
                    .fallback(#[allow(unused_braces)] { move | | View::new(
    (
            IntoMaybeErased::into_maybe_erased(blog_interface::html::p()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("正在加载…",
            "Loading…") }) }).role("status")),
            IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("取消", "Cancel") }) })
            .class("blog-button").on(blog_interface::ev::click, move | _ |
            navigate("articles")))
        ,),
) })
                    .children({ ::blog_interface::children::ToChildren::to_children(move ||
                    (IntoMaybeErased::into_maybe_erased(blog_interface::html::form()
                    .child(#[allow(unused_braces)] {
                    blog_interface::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ControlledField, { let mut props =
                    component_props_builder(& ControlledField)
                    .label(#[allow(unused_braces)] { t("标题", "Title") })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | form
                    .get().article_title) }).change(#[allow(unused_braces)] {
                    Callback::new(move | v : String | form.update(| f | f
                    .article_title = v)) }).required(#[allow(unused_braces)] { true
                    }).max(#[allow(unused_braces)] { 120 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("分类", "Category") })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & crate ::select::Select, { let mut props =
                    component_props_builder(& crate
                    ::select::Select).label(#[allow(unused_braces)] {
                    Signal::derive(| | t("分类", "Category")) })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | form
                    .get().sort_id.to_string()) }).options(#[allow(unused_braces)] {
                    Signal::derive(move | | std::iter::once(("0".to_string(),
                    t("选择分类", "Select a category"))).chain(categories.value
                    .get().unwrap_or_default().into_iter().map(| c | (c.id
                    .to_string(), c.sort_name))).collect()) })
                    .required(#[allow(unused_braces)] { true })
                    .change(#[allow(unused_braces)] { Callback::new(move | v : String
                    | form.update(| f | { f.sort_id = v.parse:: < i64 > ()
                    .unwrap_or(0); f.label_id = 0; })) }).build(); props }) }),))
                    .class("blog-form-field")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("标签", "Tags") })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & crate ::select::Select, { let mut props =
                    component_props_builder(& crate
                    ::select::Select).label(#[allow(unused_braces)] {
                    Signal::derive(| | t("标签", "Tags")) })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | form
                    .get().label_id.to_string()) }).options(#[allow(unused_braces)] {
                    Signal::derive(move | | std::iter::once(("0".to_string(),
                    t("选择标签", "Select a tag"))).chain(labels.value.get()
                    .unwrap_or_default().into_iter().filter(| c | c.sort_id == form
                    .get().sort_id).map(| c | (c.id.to_string(), c.label_name)))
                    .collect()) }).required(#[allow(unused_braces)] { true })
                    .change(#[allow(unused_braces)] { Callback::new(move | v : String
                    | form.update(| f | f.label_id = v.parse:: < i64 > ()
                    .unwrap_or(0))) }).build(); props }) }),))
                    .class("blog-form-field")),)).class("form-grid")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ControlledField, { let mut props =
                    component_props_builder(& ControlledField)
                    .label(#[allow(unused_braces)] { t("封面图片地址",
                    "Cover image URL") }).value(#[allow(unused_braces)] {
                    Signal::derive(move | | form.get().article_cover
                    .unwrap_or_default()) }).change(#[allow(unused_braces)] {
                    Callback::new(move | v : String | form.update(| f | f
                    .article_cover = Some(v))) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & FilePicker, { let mut props =
                    component_props_builder(& FilePicker)
                    .label(#[allow(unused_braces)] { t("上传封面图片",
                    "Upload cover image") }).disabled(#[allow(unused_braces)] {
                    Signal::derive(move | | save.busy.get() || upload.busy.get()) })
                    .choose(#[allow(unused_braces)] { Callback::new(move | file |
                    upload_file(file, true)) }).build(); props }) }),))
                    .class("form-grid")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("Markdown 正文", "Markdown content") })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::textarea()
                    .class("blog-input editor-textarea").required(true).prop("value",
                    move | | form.get()
                    .article_content).on(blog_interface::ev::input, move
                    | ev | form.update(| f | f.article_content = event_target_value(&
                    ev)))),)).class("blog-form-field")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if preview.get() { t("收起预览", "Hide preview") }
                    else { t("预览正文", "Preview content") } }) })
                    .class("blog-button").r#type("button")
                    .on(blog_interface::ev::click, move | _ | preview
                    .update(| v | * v = ! * v))),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & FilePicker, { let mut props =
                    component_props_builder(& FilePicker)
                    .label(#[allow(unused_braces)] { t("插入正文图片",
                    "Insert content image") }).disabled(#[allow(unused_braces)] {
                    Signal::derive(move | | save.busy.get() || upload.busy.get()) })
                    .choose(#[allow(unused_braces)] { Callback::new(move | file |
                    upload_file(file, false)) }).build(); props }) }),))
                    .class("blog-actions")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | preview.get().then(| | View::new(
    blog_interface::html::div()
        .class("editor-markdown-preview article-markdown")
        .inner_html(
            ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                crate::browser::markdown_html(&form.get().article_content),
            ),
        ),
))
                    })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ControlledField, { let mut props =
                    component_props_builder(& ControlledField)
                    .label(#[allow(unused_braces)] { t("视频地址", "Video URL")
                    }).value(#[allow(unused_braces)] { Signal::derive(move | | form
                    .get().video_url.unwrap_or_default()) })
                    .change(#[allow(unused_braces)] { Callback::new(move | v : String
                    | form.update(| f | f.video_url = Some(v))) }).build(); props })
                    }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! form.get().view_status).then(| | View::new(
    blog_interface::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("文章访问密码", "Article access password") })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::input()
                    .class("blog-input").r#type("password")
                    .aria_describedby(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | crate ::dates::password_error(& form.get().password
                    .unwrap_or_default(), existing.get()).map(| _ |
                    "blog-article-password-error")))
                    .aria_invalid(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | crate ::dates::password_error(& form.get().password
                    .unwrap_or_default(), existing.get()).is_some().to_string()))
                    .placeholder(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | if existing.get() { t("留空则保留原密码",
                    "Leave empty to keep the current password") } else {
                    t("至少 12 字节", "At least 12 bytes") })).prop("value",
                    move | | form.get().password
                    .unwrap_or_default()).on(blog_interface::ev::input,
                    move | ev | form.update(| f | f.password =
                    Some(event_target_value(& ev))))),)).class("blog-form-field")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | crate ::dates::password_error(& form.get().password
                    .unwrap_or_default(), existing.get()).map(| e | View::new(
    blog_interface::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ e }) },
        )
        .id("blog-article-password-error")
        .role("alert"),
))
                    }))
                ,),
        ),
)) })),)).class("form-grid")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::input()
                    .r#type("checkbox").prop("checked",
                    move | | form.get()
                    .view_status).on(blog_interface::ev::change, move |
                    ev | { let v = event_target_checked(& ev); form.update(| f | { f
                    .view_status = v; if v { f.password = None; } }); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t(" 公开访问（关闭后需设置密码）",
                    " Public access (a password is required when disabled)") })),))),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ControlledCheck, { let mut props =
                    component_props_builder(& ControlledCheck)
                    .label(#[allow(unused_braces)] { t(" 推荐", " Recommended") })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | form
                    .get().recommend_status) }).change(#[allow(unused_braces)] {
                    Callback::new(move | v | form.update(| f | f.recommend_status =
                    v)) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ControlledCheck, { let mut props =
                    component_props_builder(& ControlledCheck)
                    .label(#[allow(unused_braces)] { t(" 允许评论",
                    " Allow comments") }).value(#[allow(unused_braces)] {
                    Signal::derive(move | | form.get().comment_status) })
                    .change(#[allow(unused_braces)] { Callback::new(move | v | form
                    .update(| f | f.comment_status = v)) }).build(); props }) }),))
                    .class("check-row")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! form.get().view_status).then(| | View::new(
    (
            IntoMaybeErased::into_maybe_erased({
            #[allow(unreachable_code)] #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
            & ControlledField, { let mut props =
            component_props_builder(& ControlledField)
            .label(#[allow(unused_braces)] { t("密码提示", "Password hint") })
            .value(#[allow(unused_braces)] { Signal::derive(move | | form.get().tips
            .unwrap_or_default()) }).change(#[allow(unused_braces)] {
            Callback::new(move | v : String | form.update(| f | f.tips = Some(v))) })
            .build(); props }) }),
            IntoMaybeErased::into_maybe_erased(blog_interface::html::span()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({
            t("显示在文章访问密码输入页。",
            "Displayed on the article access password screen.") }) })
            .class("blog-field-help"))
        ,),
)) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("取消",
                    "Cancel") }) }).class("blog-button").r#type("button")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())).on(blog_interface::ev::click,
                    move | _ | { upload.abort(); navigate("articles") })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if upload.busy.get() { t("图片上传中…",
                    "Uploading image…") } else if save.busy.get() {
                    t("保存中…", "Saving…") } else { t("保存文章",
                    "Save article") } }) }).class("blog-button").r#type("submit")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || upload.busy.get()))),))
                    .class("blog-actions")),)).class("blog-fieldset")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())) }).class("editor-form")
                    .on(blog_interface::ev::submit, move | ev | { ev
                    .prevent_default(); if upload.busy.get_untracked() || ! ready
                    .get_untracked() { return; } let f = form.get_untracked(); let
                    issue = if f.view_status { None } else { crate
                    ::dates::password_error(f.password.as_deref()
                    .unwrap_or_default(), existing.get_untracked()) }; if let
                    Some(issue) = issue { validation.set(issue); return; } let f = f
                    .for_submission(); let Ok(body) =
                    blog_interface::api::encode_json_body(& f) else { save.error
                    .set(Some(invalid())); return; }; let target = if id > 0 {
                    Some(id) } else { created_id.get_value() }; let path = target
                    .map(| id | format!("/api/v1/content/articles/{id}"))
                    .unwrap_or("/api/v1/content/articles".into()); let opts =
                    RequestOptions { headers : if target.is_none() {
                    vec![("Idempotency-Key".into(), creation.get_value())] } else {
                    vec![] }, ..Default::default() }; save.run_typed_options(path, if
                    target.is_some() { "PUT" } else { "POST" }, Some(body), opts,
                    move | v : Article | { if id == 0 { created_id.set_value(Some(v
                    .id)); } notify(t("文章已保存", "Article saved"));
                    navigate("articles"); }); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    (id > 0).then(| | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &NewsEditor,
        {
            let mut props = component_props_builder(&NewsEditor)
                .id(#[allow(unused_braces)] { id })
                .build();
            props
        },
    )
})) })),)) })
                    .build(); props }) })
                ,),
        )
        .class("blog-content-stack"),
)
}
#[component]
fn NewsEditor(id: i64) -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load_typed::<Page<News>>(Signal::derive(move || {
        format!(
            "/api/v1/content/articles/{id}/news?page={}&size=20",
            page.get()
        )
    }));
    let form = RwSignal::new(json!({"content":"","create_time":""}));
    let original = RwSignal::new(json!({"content":"","create_time":""}));
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let receipt = StoredValue::new(None::<(String, String)>);
    let validation = RwSignal::new(String::new());
    let reset = Callback::new(move |()| {
        let empty = json!({"content":"","create_time":""});
        form.set(empty.clone());
        original.set(empty);
        editing.set(None);
        receipt.set_value(None);
        validation.set(String::new());
    });
    View::new(
    blog_interface::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("最新进展",
                    "Recent updates") }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .fallback(#[allow(unused_braces)] { t("文章进展加载失败",
                    "Unable to load article updates") })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error }).build(); props })
                    }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! validation.get().is_empty()).then(| | View::new(
    blog_interface::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ validation.get() }) },
        )
        .role("alert"),
)) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::form()
                    .child(#[allow(unused_braces)] {
                    blog_interface::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if editing.get().is_some() { t("编辑进展",
                    "Edit update") } else { t("新增进展", "Add update") } })
                    })), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("进展内容",
                    "Update content") }).key(#[allow(unused_braces)] { "content" })
                    .form(#[allow(unused_braces)] { form })
                    .textarea(#[allow(unused_braces)] { true })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 2048 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("进展时间（UTC）", "Update time (UTC)") })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::input()
                    .class("blog-input").r#type("datetime-local").step("1")
                    .prop("value", move | | text(&
                    form.get(), "create_time"))
                    .on(blog_interface::ev::input, move | ev | form
                    .update(| f | f["create_time"] = json!(event_target_value(&
                    ev))))),)).class("blog-form-field")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("新增时留空使用当前 UTC；编辑时留空保留原时间。",
                    "Leave blank to use current UTC for a new update, or keep its existing time when editing.")
                    }) }).class("blog-field-help")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if editing.get().is_some() { t("保存修改",
                    "Save changes") } else { t("发布进展", "Publish update") }
                    }) }).class("blog-button").r#type("submit")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (editing.get().is_some() || form.get() != original
                    .get()).then(| | View::new(
    blog_interface::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("取消编辑", "Cancel editing")
                })
            },
        )
        .class("blog-button")
        .r#type("button")
        .on(
            blog_interface::ev::click,
            move |_| {
                if form.get_untracked() == original.get_untracked()
                    || web_sys::window()
                        .unwrap()
                        .confirm_with_message(
                            &t(
                                "放弃当前未保存的进展修改？",
                                "Discard the current unsaved update changes?",
                            ),
                        )
                        .unwrap_or(false)
                {
                    reset.run(())
                }
            },
        ),
)) })),)).class("blog-actions")),))
                    .class("blog-fieldset")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || data.value.get().is_none())) })
                    .class("editor-form").on(blog_interface::ev::submit,
                    move | ev | { ev.prevent_default(); let f = form.get_untracked();
                    let content = text(& f, "content"); if ! (1..= 1024).contains(&
                    content.trim().chars().count()) { validation
                    .set(t("进展正文须为 1–1024 个字符",
                    "Update content must contain 1–1024 characters")); return; }
                    let input = json!({ "content" : content, "create_time" : crate
                    ::dates::iso(& text(& f, "create_time")) }); let body = input
                    .to_string(); if editing.get_untracked().is_none() && receipt
                    .get_value().as_ref().is_none_or(| (_, b) | * b != body) {
                    receipt.set_value(Some((web_sys::window().unwrap().crypto()
                    .unwrap().random_uuid(), body))); } let target = editing
                    .get_untracked(); let path =
                    format!("/api/v1/content/articles/{id}/news{}", target.map(| id |
                    format!("/{id}")).unwrap_or_default()); let opts = RequestOptions
                    { headers : if target.is_none() { receipt.get_value().map(| (k,
                    _) | vec![("Idempotency-Key".into(), k)]).unwrap_or_default() }
                    else { vec![] }, ..Default::default() }; save.run_options(path,
                    if target.is_some() { "PUT" } else { "POST" }, Some(input), opts,
                    move | v | { if text(& v, "content") != content.trim() || target
                    .is_some_and(| i | num(& v, "id") != i) { save.error
                    .set(Some(invalid())); return; } reset.run(()); page.set(1); data
                    .reload(); notify(t("文章进展已保存",
                    "Article update saved")); }); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().map(| v | View::new(
    blog_interface::html::div()
        .child(
            #[allow(unused_braces)]
            {
                blog_interface::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(blog_interface::html::thead()
                                .child(#[allow(unused_braces)] {
                                blog_interface::html::tr()
                                .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("时间",
                                "Time") }) })),
                                IntoMaybeErased::into_maybe_erased(blog_interface::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("内容",
                                "Content") }) })),
                                IntoMaybeErased::into_maybe_erased(blog_interface::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("操作",
                                "Actions") }) })),)) })),
                                IntoMaybeErased::into_maybe_erased(blog_interface::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ v.items
                                .into_iter().map(| item | { let entry_id = item.id; let edit
                                = item.clone(); View::new(
    blog_interface::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ crate
                    ::dates::format(& item.create_time.unwrap_or_default()) }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ item.content }) })
                    .class("comment-cell")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("编辑", "Edit")
                    }) }).class("blog-button")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())).on(blog_interface::ev::click,
                    move | _ | { if form.get_untracked() != original.get_untracked()
                    && ! web_sys::window().unwrap().confirm_with_message(&
                    t("放弃当前未保存的进展修改？",
                    "Discard the current unsaved update changes?")).unwrap_or(false)
                    { return; } let f = json!({ "content" : edit.content,
                    "create_time" : crate ::dates::input(& edit.create_time.clone()
                    .unwrap_or_default()) }); form.set(f.clone()); original.set(f);
                    editing.set(Some(entry_id)); })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("删除",
                    "Delete") }) }).class("blog-button blog-danger")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())).on(blog_interface::ev::click,
                    move | _ | { if web_sys::window().unwrap().confirm_with_message(&
                    t("确定删除这条文章进展？",
                    "Delete this article update?")).unwrap_or(false) { save
                    .run(format!("/api/v1/content/articles/{id}/news/{entry_id}"),
                    "DELETE", None, move | _ | { if editing.get_untracked() ==
                    Some(entry_id) { reset.run(()) } data.reload(); }); } })),))
                    .class("blog-actions") }))
                ,),
        ),
)
                                }).collect_view() }) }))
                            ,),
                    )
                    .class("blog-table")
                    .aria_label(
                        ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                            t("文章进展", "Article updates"),
                        ),
                    )
            },
        )
        .class("blog-table-scroll")
        .tabindex("0")
        .role("region")
        .aria_label(
            ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                t("数据表格", "Data table"),
            ),
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Pager, { let mut props =
                    component_props_builder(& Pager)
                    .data(#[allow(unused_braces)] { data })
                    .page(#[allow(unused_braces)] { page })
                    .label(#[allow(unused_braces)] { t("进展分页",
                    "Update pages") }).build(); props }) })
                ,),
        )
        .class("editor-form"),
)
}
fn empty_note() -> Value {
    json!({"content":"","image_path":null,"is_public":true})
}
#[component]
pub fn Notes() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
        format!("/api/v1/content/notes?page={}&size=20", page.get())
    }));
    let form = RwSignal::new(empty_note());
    let original = RwSignal::new(empty_note());
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let upload = Action::new();
    let filename = RwSignal::new(String::new());
    let lightbox = RwSignal::new(None::<String>);
    let validation = RwSignal::new(String::new());
    let reset = Callback::new(move |()| {
        upload.abort();
        form.set(empty_note());
        original.set(empty_note());
        editing.set(None);
        filename.set(String::new());
    });
    View::new(
    blog_interface::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("随笔管理",
                    "Notes") }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("维护随笔及恋爱日常中的内容。",
                    "Manage notes shown in the journal and everyday moments.") })
                    })), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .fallback(#[allow(unused_braces)] { t("随笔加载失败",
                    "Unable to load notes") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | data.reload())) }).build(); props
                    }) }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error }).build(); props })
                    }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { upload.error }).build(); props
                    }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! validation.get().is_empty()).then(| | View::new(
    blog_interface::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ validation.get() }) },
        )
        .role("alert"),
)) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::form()
                    .child(#[allow(unused_braces)] {
                    blog_interface::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if editing.get().is_some() { t("编辑随笔",
                    "Edit note") } else { t("新增随笔", "Add note") } }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("随笔正文",
                    "Note content") }).key(#[allow(unused_braces)] { "content" })
                    .form(#[allow(unused_braces)] { form })
                    .textarea(#[allow(unused_braces)] { true })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 2048 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & FilePicker, { let mut props =
                    component_props_builder(& FilePicker)
                    .label(#[allow(unused_braces)] { t("随笔图片", "Note image")
                    }).filename(#[allow(unused_braces)] { Signal::derive(move | |
                    filename.get()) }).disabled(#[allow(unused_braces)] {
                    Signal::derive(move | | upload.busy.get() || save.busy.get()) })
                    .choose(#[allow(unused_braces)] { Callback::new(move | file :
                    blog_interface::thread_bound::SendWrapper < web_sys::File > | { let name = file
                    .name(); upload.upload(file, move | v | { form.update(| f |
                    f["image_path"] = v["path"].clone()); filename.set(name);
                    notify(t("图片已上传", "Image uploaded")); }); }) })
                    .build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | safe_url(& text(& form.get(), "image_path"), true).map(|
                    url | { let click = url.clone(); View::new(
    blog_interface::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    blog_interface::html::img()
                    .src(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(url))
                    .alt(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(t("查看随笔图片",
                    "View note image"))) }).class("moderation-photo")
                    .r#type("button").on(blog_interface::ev::click, move |
                    _ | lightbox.set(Some(click.clone())))),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("移除图片",
                    "Remove image") }) }).class("blog-button").r#type("button")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | upload.busy.get() || save.busy.get()))
                    .on(blog_interface::ev::click, move | _ | { form
                    .update(| f | f["image_path"] = Value::Null); filename
                    .set(String::new()) }))
                ,),
        ),
) }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Check, { let mut props =
                    component_props_builder(& Check)
                    .label(#[allow(unused_braces)] { t(" 公开可见",
                    " Publicly visible") }).key(#[allow(unused_braces)] { "is_public"
                    }).form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if upload.busy.get() { t("图片上传中…",
                    "Uploading image…") } else if save.busy.get() {
                    t("保存中…", "Saving…") } else if editing.get().is_some()
                    { t("保存修改", "Save changes") } else { t("发布",
                    "Publish") } }) }).class("blog-button").r#type("submit")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || upload.busy.get()))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (editing.get().is_some() || form.get() != original.get()
                    || upload.busy.get()).then(| | View::new(
    blog_interface::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("取消编辑", "Cancel editing")
                })
            },
        )
        .class("blog-button")
        .r#type("button")
        .disabled(
            ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                move || save.busy.get(),
            ),
        )
        .on(
            blog_interface::ev::click,
            move |_| {
                if form.get_untracked() == original.get_untracked()
                    || web_sys::window()
                        .unwrap()
                        .confirm_with_message(
                            &t(
                                "放弃当前未保存的随笔修改？",
                                "Discard the current unsaved note changes?",
                            ),
                        )
                        .unwrap_or(false)
                {
                    reset.run(())
                }
            },
        ),
)) })),)).class("blog-actions")),))
                    .class("blog-fieldset")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())) }).class("editor-form")
                    .on(blog_interface::ev::submit, move | ev | { ev
                    .prevent_default(); if upload.busy.get_untracked() { return; }
                    let f = form.get_untracked(); let content = text(& f, "content");
                    if ! (1..= 1024).contains(& content.trim().chars().count()) {
                    validation.set(t("随笔正文须为 1–1024 个字符",
                    "Note content must contain 1–1024 characters")); return; } let
                    id = editing.get_untracked(); let expected = f.clone(); let path
                    = id.map(| id | format!("/api/v1/content/notes/{id}"))
                    .unwrap_or("/api/v1/content/notes".into()); save.run(path, if id
                    .is_some() { "PUT" } else { "POST" }, Some(f), move | v | { if
                    text(& v, "content") != content.trim() || v["image_path"] !=
                    expected["image_path"] || flag(& v, "is_public") != flag(&
                    expected, "is_public") || id.is_some_and(| id | num(& v, "id") !=
                    id) { save.error.set(Some(invalid())); return; } reset.run(());
                    page.set(1); data.reload(); notify(t("随笔已保存",
                    "Note saved")); }); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().map(| v | View::new(
    blog_interface::html::div()
        .child(
            #[allow(unused_braces)]
            {
                blog_interface::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(blog_interface::html::thead()
                                .child(#[allow(unused_braces)] {
                                blog_interface::html::tr()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ [t("作者",
                                "Author"), t("内容", "Content"), t("图片", "Image"),
                                t("状态", "Status"), t("时间", "Time"), t("操作",
                                "Actions")].into_iter().map(| l | View::new(
    blog_interface::html::th()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        ),
)).collect_view() }) }) })),
                                IntoMaybeErased::into_maybe_erased(blog_interface::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ rows(& v,
                                "items").into_iter().map(| item | { let id = num(& item,
                                "id"); let edit = item.clone(); let toggle = item.clone();
                                View::new(
    blog_interface::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ let n = text(& item,
                    "username"); if n.is_empty() { t("站长", "Site owner") } else {
                    n } }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item,
                    "content") }) }).class("comment-cell")),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ safe_url(& text(&
                    item, "image_path"), true).map(| u | { let click = u.clone();
                    View::new(
    blog_interface::html::button()
        .child(
            #[allow(unused_braces)]
            {
                blog_interface::html::img()
                    .src(
                        ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                            u,
                        ),
                    )
                    .alt(
                        ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                            t("查看随笔图片", "View note image"),
                        ),
                    )
            },
        )
        .class("moderation-photo")
        .r#type("button")
        .on(
            blog_interface::ev::click,
            move |_| lightbox.set(Some(click.clone())),
        ),
) }) }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if flag(& item,
                    "is_public") { t("公开", "Public") } else { t("私密",
                    "Private") } }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ table_time(& text(&
                    item, "create_time")) }) })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::td()
                    .child(#[allow(unused_braces)] {
                    blog_interface::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("编辑", "Edit")
                    }) }).class("blog-button")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || upload.busy.get()))
                    .on(blog_interface::ev::click, move | _ | { if form
                    .get_untracked() != original.get_untracked() && !
                    web_sys::window().unwrap().confirm_with_message(&
                    t("放弃当前未保存的随笔修改？",
                    "Discard the current unsaved note changes?")).unwrap_or(false) {
                    return; } let f = json!({ "content" : edit["content"],
                    "image_path" : edit["image_path"], "is_public" : flag(& edit,
                    "is_public") }); form.set(f.clone()); original.set(f); editing
                    .set(Some(id)); filename.set(String::new()); })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if flag(& item,
                    "is_public") { t("隐藏", "Hide") } else { t("公开",
                    "Publish") } }) }).class("blog-button")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || upload.busy.get() || editing.get() ==
                    Some(id) && form.get() != original.get()))
                    .on(blog_interface::ev::click, move | _ | { let
                    visible = ! flag(& toggle, "is_public"); let input = json!({
                    "content" : toggle["content"], "image_path" :
                    toggle["image_path"], "is_public" : visible }); save
                    .run(format!("/api/v1/content/notes/{id}"), "PUT", Some(input),
                    move | v | { if editing.get_untracked() == Some(id) { form
                    .update(| f | f["is_public"] = json!(visible)); original.update(|
                    f | f["is_public"] = json!(visible)); }
                    if num(& v, "id") != id
                    || flag(& v, "is_public") != visible { save.error
                    .set(Some(invalid())); return; } data.reload(); }); })),
                    IntoMaybeErased::into_maybe_erased(blog_interface::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("删除",
                    "Delete") }) }).class("blog-button blog-danger")
                    .disabled(::blog_interface::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || upload.busy.get()))
                    .on(blog_interface::ev::click, move | _ | { if
                    web_sys::window().unwrap().confirm_with_message(&
                    t("确定删除这条随笔？", "Delete this note?"))
                    .unwrap_or(false) { save
                    .run(format!("/api/v1/content/notes/{id}"), "DELETE", None, move
                    | _ | { if editing.get_untracked() == Some(id) { reset.run(()) }
                    data.reload(); }); } })),)).class("blog-actions") }))
                ,),
        ),
) }).collect_view() })
                                }))
                            ,),
                    )
                    .class("blog-table")
                    .aria_label(
                        ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                            t("随笔列表", "Note list"),
                        ),
                    )
            },
        )
        .class("blog-table-scroll")
        .tabindex("0")
        .role("region")
        .aria_label(
            ::blog_interface::prelude::IntoAttributeValue::into_attribute_value(
                t("数据表格", "Data table"),
            ),
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ListState, { let mut props =
                    component_props_builder(& ListState)
                    .data(#[allow(unused_braces)] { data })
                    .empty(#[allow(unused_braces)] { t("暂无随笔",
                    "No notes yet") }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Pager, { let mut props =
                    component_props_builder(& Pager)
                    .data(#[allow(unused_braces)] { data })
                    .page(#[allow(unused_braces)] { page }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Lightbox, { let mut props =
                    component_props_builder(& Lightbox)
                    .src(#[allow(unused_braces)] { lightbox }).build(); props }) })
                ,),
        )
        .class("blog-content-stack"),
)
}
