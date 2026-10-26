use crate::common::{
    Action, Feedback, FilePicker, Lightbox, ListState, Pager, load, nullable_num, num, rows,
    safe_url, t, table_time, text, tr,
};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ModerationKind {
    Comments,
    Messages,
    Files,
}

impl ModerationKind {
    fn endpoint(self) -> &'static str {
        match self {
            Self::Comments => "comments",
            Self::Messages => "tree-hole",
            Self::Files => "resources",
        }
    }
}

fn moderation_author(item: &Value, message: bool) -> String {
    let username = text(item, "username");
    if !username.is_empty() {
        return username;
    }
    if let Some(user) = nullable_num(item, "user_id") {
        if message {
            tr("用户 {0}", "User {0}", &[user.to_string()])
        } else {
            user.to_string()
        }
    } else {
        t("匿名访客", "Anonymous guest")
    }
}

#[component]
pub(super) fn Moderation(kind: ModerationKind) -> impl IntoView {
    let endpoint = kind.endpoint();
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
        format!("/api/v1/content/{endpoint}?page={}&size=20", page.get())
    }));
    let action = Action::new();
    let upload = Action::new();
    let lightbox = RwSignal::new(None::<String>);
    let chosen = StoredValue::new(blog_interface::thread_bound::SendWrapper::new(
        None::<web_sys::File>,
    ));
    let filename = RwSignal::new(String::new());
    let uploaded = RwSignal::new(String::new());
    let title = match kind {
        ModerationKind::Comments => t("评论管理", "Comments"),
        ModerationKind::Messages => t("留言板管理", "Message board"),
        ModerationKind::Files => t("文件管理", "Files"),
    };
    let render_row = move |item: Value| {
        let id = num(&item, "id");
        let path = text(&item, "path");
        let path2 = path.clone();
        View::new(
    blog_interface::html::tr()
        .child(
            (
                    if kind == ModerationKind::Files { View::new(
    (
            blog_interface::html::td().class("blog-title-cell")
            .child( text(& item,
            "original_name")),
            blog_interface::html::td()
            .child( text(& item,
            "resource_type")),
            blog_interface::html::td()
            .child( nullable_num(& item, "size").unwrap_or(0)),
            blog_interface::html::td()
            .child( safe_url(& path, false)
            .map(| u | View::new(
    blog_interface::html::a()
        .class("blog-button")
        .child(

            t("打开", "Open"),
        )
        .href(u)
        .target("_blank")
        .rel("noopener noreferrer"),
)))
        ,),
) .into_any() } else if kind == ModerationKind::Comments {
                    View::new(
    (
            blog_interface::html::td()
            .child( match text(& item,
            "comment_type").as_str() { "message" => t("留言板", "Message board")
            .into_any(), "love" => View::new(
    blog_interface::html::a()
        .class("blog-button")
        .child(

            t("祝福板", "Wishes"),
        )
        .href("/love")
        .target("_blank")
        .rel("noopener noreferrer"),
)
            .into_any(), _ => View::new(
    blog_interface::html::a()
        .class("blog-button")
        .child(

            format!(
                        "{}{}", t("文章 #", "Article #"), num(& item, "source")
                    ),
        )
        .href(
            format!("/article/{}", num(& item, "source")),
        )
        .target("_blank")
        .rel("noopener noreferrer"),
) .into_any() }),
            blog_interface::html::td()
            .child( moderation_author(& item,
            false)),
            blog_interface::html::td()
            .child( text(& item,
            "comment_content")).class("comment-cell"),
            blog_interface::html::td()
            .child( table_time(& text(& item,
            "create_time")))
        ,),
)
                    .into_any() } else { View::new(
    (
            blog_interface::html::td()
            .child( id),
            blog_interface::html::td()
            .child( moderation_author(& item,
            true)),
            blog_interface::html::td().class("comment-cell")
            .child( { let message = text(& item,
            "message"); if message.is_empty() { t("图片留言", "Image message") }
            else { message } }),
            blog_interface::html::td()
            .child( safe_url(& text(& item,
            "image_path"), true).map(| u | { let click = u.clone(); View::new(
    blog_interface::html::button()
        .child(

            blog_interface::html::img()
                    .src(
                        u,
                    )
                    .alt(
                        t("查看留言图片", "View message image"),
                    ),
        )
        .class("moderation-photo")
        .r#type("button")
        .on(
            blog_interface::ev::click,
            move |_| lightbox.set(Some(click.clone())),
        ),
) })),
            blog_interface::html::td()
            .child( table_time(& text(& item,
            "create_time")))
        ,),
) .into_any() },
                    blog_interface::html::td()
                    .child( blog_interface::html::button()
                    .child( t("删除",
                    "Delete")).class("blog-button blog-danger")
                    .disabled(move
                    | | action.busy.get() || upload.busy.get() || (kind ==
                    ModerationKind::Files && path2.is_empty()))
                    .on(blog_interface::ev::click, move | _ | { if !
                    web_sys::window().expect("invariant: the application is mounted in a browser window").confirm_with_message(& match kind {
                    ModerationKind::Comments =>
                    t("删除这条评论？如果是顶层评论，会同时删除该楼层的全部回复。",
                    "Delete this comment? If it is a top-level comment, all replies in its thread will also be deleted."),
                    ModerationKind::Files => t("确定删除这个文件及其媒体数据？",
                    "Delete this file and its media data?"), ModerationKind::Messages =>
                    t("确定删除这条留言？", "Delete this message?") })
                    .unwrap_or(false) { return; } let body = (kind == ModerationKind::Files)
                    .then(| | json!({ "path" : path })); action
                    .run(format!("/api/v1/content/{endpoint}/{id}"), "DELETE", body, move
                    | _ | data.reload()); }))
                ,),
        ),
)
    };
    View::new(
    blog_interface::html::section()
        .child(
            (
                    blog_interface::html::h1().class("blog-row")
                    .child( title),
                    Feedback(component_props_builder(& Feedback)
                    .error( data.error)
                    .retry( Some(Callback::new(move | () |
                    data.reload()))).build()),
                    Feedback(component_props_builder(& Feedback)
                    .error( action.error).build()), Feedback(component_props_builder(& Feedback)
                    .error( upload.error).build()),
                    blog_interface::html::div()
                    .class("blog-stack blog-stack--spaced")
                    .child((
                    (kind == ModerationKind::Files).then(| | View::new(
    (
            blog_interface::html::fieldset()
            .child( blog_interface::html::form()
            .child((FilePicker(component_props_builder(& FilePicker)
            .label( t("上传图片（PNG、JPEG、GIF、WebP）",
            "Upload image (PNG, JPEG, GIF, WebP)"))
            .filename( Signal::derive(move | | filename
            .get())).disabled( Signal::derive(move | |
            upload.busy.get() || action.busy.get())).choose(
            Callback::new(move | f : blog_interface::thread_bound::SendWrapper < web_sys::File > |
            { filename.set(f.name()); chosen
            .set_value(blog_interface::thread_bound::SendWrapper::new(Some((* f).clone()))); }))
            .build()),
            blog_interface::html::button()
            .child( move | | if upload.busy.get() { t("上传中…", "Uploading…") } else { t("上传", "Upload") }).class("blog-button").r#type("submit")
            .disabled(move
            | | filename.get().is_empty() || upload.busy.get()),))
            .class("inline-form blog-frame blog-frame--padded blog-flow blog-form-flow").on(blog_interface::ev::submit, move | ev
            | { ev.prevent_default(); let file = chosen.with_value(| f | f.as_ref()
            .cloned()); if let Some(file) = file { upload
            .upload(blog_interface::thread_bound::SendWrapper::new(file), move | v | { uploaded
            .set(text(& v, "path")); filename.set(String::new()); chosen
            .set_value(blog_interface::thread_bound::SendWrapper::new(None)); data.reload();
            notify(t("图片已上传", "Image uploaded")); }) } }))
            .class("blog-fieldset blog-stack")
            .disabled(move
            | | upload.busy.get() || action.busy.get()),
            move | | (! uploaded.get().is_empty()).then(| | View::new(
    blog_interface::html::p().class("blog-row")
        .child(
            (
                    t("新图片地址：", "New image URL: "),
                    blog_interface::html::code()
                    .child( uploaded.get())
                ,),
        ),
))
        ,),
)),
                    move | | data.value.get().map(| v | View::new(
    blog_interface::html::div()
        .child(

            blog_interface::html::table()
                    .child(
                        (
                                blog_interface::html::thead()
                                .child( blog_interface::html::tr()
                                .child( (if kind ==
                                ModerationKind::Comments { vec![t("来源", "Source"), t("用户",
                                "Users"), t("内容", "Content"), t("时间", "Time"),
                                t("操作", "Actions")] } else if kind == ModerationKind::Messages {
                                vec!["ID".into(), t("作者", "Author"), t("内容",
                                "Content"), t("图片", "Image"), t("时间", "Time"),
                                t("操作", "Actions")] } else { vec![t("名称", "Name"),
                                t("类型", "Type"), t("大小", "Size"), t("位置",
                                "Location"), t("操作", "Actions")] }).into_iter().map(| v
                                | View::new(
    blog_interface::html::th()
        .child(

            v,
        ),
)).collect_view())),
                                blog_interface::html::tbody()
                                .child( rows(& v,
                                "items").into_iter().map(render_row).collect_view())
                            ,),
                    )
                    .class("blog-table")
                    .aria_label(
                        match kind {
                                ModerationKind::Comments => t("评论列表", "Comment list"),
                                ModerationKind::Messages => t("留言列表", "Message list"),
                                ModerationKind::Files => t("文件列表", "File list"),
                            },
                    ),
        )
        .class("blog-table-scroll blog-frame")
        .tabindex("0")
        .role("region")
        .aria_label(
            t("数据表格", "Data table"),
        ),
)), ListState(component_props_builder(& ListState)
                    .data( data)
                    .empty( match kind { ModerationKind::Comments =>
                    t("暂无评论", "No comments yet"), ModerationKind::Messages =>
                    t("暂无留言", "No messages yet"), ModerationKind::Files => t("暂无文件",
                    "No files yet") }).build()),
                    Pager(component_props_builder(& Pager)
                    .data( data)
                    .page( page).build()),
                    )),
                    Lightbox(component_props_builder(& Lightbox)
                    .src( lightbox).build())
                ,),
        )
        .class("blog-content-stack blog-stack"),
)
}
