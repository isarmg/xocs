use crate::common::Action;
use crate::common::*;
use leptos::prelude::*;
use serde_json::{Value, json};
use xcss_web::components::{AccountPage, AdminShell, HeaderActions, HeaderNavigation, notify};
#[derive(Clone)]
struct Group {
    id: &'static str,
    label: String,
    title: String,
    pages: Vec<(&'static str, String)>,
}
fn groups() -> Vec<Group> {
    vec![
        Group {
            id: "overview",
            label: t("总览", "Overview"),
            title: t("总览", "Overview"),
            pages: vec![("dashboard", t("总览", "Overview"))],
        },
        Group {
            id: "content",
            label: t("内容", "Content"),
            title: t("内容管理", "Content management"),
            pages: vec![
                ("articles", t("文章管理", "Articles")),
                ("categories", t("分类管理", "Categories")),
                ("labels", t("标签管理", "Tags")),
                ("notes", t("随笔管理", "Notes")),
                ("letter", t("信笺管理", "Letter")),
            ],
        },
        Group {
            id: "community",
            label: t("社区", "Community"),
            title: t("社区管理", "Community management"),
            pages: vec![
                ("comments", t("评论管理", "Comments")),
                ("tree-hole", t("留言板管理", "Message board")),
            ],
        },
        Group {
            id: "resources",
            label: t("资源", "Resources"),
            title: t("资源管理", "Resource management"),
            pages: vec![
                ("resources", t("文件管理", "Files")),
                ("links", t("内容资源", "Content resources")),
                ("family", t("恋爱笔记", "Love journal")),
            ],
        },
        Group {
            id: "settings",
            label: t("设置", "Settings"),
            title: t("网站设置", "Site settings"),
            pages: vec![
                ("site", t("网站设置", "Site settings")),
                ("home", t("首页栏目", "Home sections")),
            ],
        },
    ]
}
#[component]
pub fn Admin() -> impl IntoView {
    view! {<div class="xocs-admin-app"><AdminShell product="xocs" landing="/admin#articles"><Workspace/></AdminShell></div>}
}
#[component]
fn Workspace() -> impl IntoView {
    let route = hash_signal();
    let active = Signal::derive(move || {
        let r = route.get();
        if r == "new" || r.starts_with("edit/") || r.is_empty() {
            "articles".into()
        } else {
            r
        }
    });
    let group = Signal::derive(move || {
        groups()
            .into_iter()
            .find(|g| g.pages.iter().any(|(p, _)| *p == active.get()))
            .map(|g| g.id)
            .unwrap_or("content")
    });
    view! {<div class="xocs-admin-workspace xcss-content-stack"><HeaderNavigation label=t("后台导航","Administration navigation")>{groups().into_iter().map(|g|view!{<button class="xcss-button" aria-pressed=move||(group.get()==g.id&&active.get()!="account").to_string() on:click=move |_|navigate(g.pages[0].0)>{g.label}</button>}).collect_view()}</HeaderNavigation><HeaderActions><a class="xocs-site-link" href=href("/") aria-label=t("返回网站","Back to site") title=t("返回网站","Back to site")><svg aria-hidden="true" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="m3 10 9-7 9 7M5 9v11h5v-7h4v7h5V9"/></svg></a></HeaderActions>{move||groups().into_iter().find(|g|g.id==group.get()&&g.pages.len()>1&&active.get()!="account").map(|g|view!{<nav class="xcss-secondary-navigation" aria-label=g.title>{g.pages.into_iter().map(|(id,label)|view!{<button class="xcss-button" aria-pressed=move||(active.get()==id).to_string() on:click=move |_|navigate(id)>{label}</button>}).collect_view()}</nav>})}{move||{let route=route.get();match route.as_str(){"account"=>view!{<AccountPage/>}.into_any(),"dashboard"=>view!{<Dashboard/>}.into_any(),"categories"=>view!{<Taxonomy labels=false/>}.into_any(),"labels"=>view!{<Taxonomy labels=true/>}.into_any(),"comments"=>view!{<Moderation kind="comments"/>}.into_any(),"tree-hole"=>view!{<Moderation kind="tree-hole"/>}.into_any(),"resources"=>view!{<Moderation kind="resources"/>}.into_any(),"site"=>view!{<SiteSettings/>}.into_any(),"letter"=>view!{<LetterSettings/>}.into_any(),"family"=>view!{<FamilySettings/>}.into_any(),"home"=>view!{<HomeSections/>}.into_any(),"links"=>view!{<Links/>}.into_any(),"notes"=>view!{<crate::editor::Notes/>}.into_any(),"new"=>view!{<crate::editor::ArticleEditor/>}.into_any(),s if s.starts_with("edit/")=>view!{<crate::editor::ArticleEditor id={s[5..].parse::<i64>().unwrap_or(0)}/>}.into_any(),_=>view!{<Articles/>}.into_any()}}}</div>}
}
#[component]
fn Dashboard() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/statistics".into()));
    view! {<section class="xcss-content-stack"><h1>{t("总览","Overview")}</h1><p>{t("网站内容与社区状态。","Site content and community status.")}</p><Feedback error=data.error fallback=t("统计信息加载失败","Unable to load statistics") retry=Some(Callback::new(move|()|data.reload()))/>{move||data.value.get().map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table xcss-statistics-table" aria-label=t("网站统计","Site statistics")><thead><tr><th>{t("指标","Metric")}</th><th>{t("数量","Count")}</th></tr></thead><tbody>{[("articles",t("文章","Articles")),("comments",t("评论","Comments")),("members",t("用户","Users")),("categories",t("分类","Categories"))].into_iter().map(|(k,l)|view!{<tr><th scope="row">{l}</th><td>{num(&v,k)}</td></tr>}).collect_view()}</tbody></table></div>})}<div class="xocs-dashboard-sections">{groups().into_iter().filter(|g|g.id!="overview").map(|g|view!{<section class="xcss-content-panel"><h2>{g.title.clone()}</h2><nav aria-label=g.title>{g.pages.into_iter().map(|(id,l)|view!{<a href=href(&format!("/admin#{id}"))>{l}</a>}).collect_view()}</nav></section>}).collect_view()}</div></section>}
}
#[component]
fn Articles() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
        format!("/api/v1/content/articles?page={}&size=15", page.get())
    }));
    let action = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let last =
                ((num(&v, "total") + num(&v, "size") - 1) / num(&v, "size").max(1)).max(1) as u64;
            if page.get_untracked() > last {
                page.set(last);
            }
        }
    });
    view! {<section class="xcss-content-stack"><HeaderActions><button class="xcss-button" aria-label=t("新增文章","Add article") title=t("新增文章","Add article") on:click=move |_|navigate("new")>"+"</button><button class="xcss-button" aria-label=t("刷新","Refresh") title=t("刷新","Refresh") disabled=move||data.busy.get() on:click=move |_|data.reload()>"↻"</button></HeaderActions><h1>{t("文章管理","Articles")}</h1><p>{t("管理公开文章与草稿。","Manage published articles and drafts.")}</p><Feedback error=data.error fallback=t("文章加载失败","Unable to load articles") retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=action.error fallback=t("删除失败，请刷新后重试","Delete failed. Refresh and try again.")/>{move||data.value.get().map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=t("文章列表","Article list")><thead><tr>{[t("标题","Title"),t("状态","Status"),t("浏览","Views"),t("创建时间","Created at"),t("操作","Actions")].into_iter().map(|l|view!{<th>{l}</th>}).collect_view()}</tr></thead><tbody>{rows(&v,"items").into_iter().map(|a|{let id=num(&a,"id");view!{<tr><td><strong>{text(&a,"article_title")}</strong><small class="muted">{format!("#{id}")}</small></td><td><span class=if flag(&a,"view_status"){"badge success"}else{"badge"}>{if flag(&a,"view_status"){t("公开","Public")}else{t("加密","Password protected")}}</span></td><td>{num(&a,"view_count")}</td><td>{crate::dates::format(&text(&a,"create_time"))}</td><td><div class="xcss-actions"><button class="xcss-button" on:click=move |_|navigate(&format!("edit/{id}"))>{t("编辑","Edit")}</button><button class="xcss-button xcss-danger" disabled=move||action.busy.get() on:click=move |_|{if web_sys::window().unwrap().confirm_with_message(&t("确定删除这篇文章？","Delete this article?")).unwrap_or(false){action.run(format!("/api/v1/content/articles/{id}"),"DELETE",None,move |_|{notify(t("文章已删除","Article deleted"));data.reload();});}}>{t("删除","Delete")}</button></div></td></tr>}}).collect_view()}</tbody></table></div>})}<Pager data page/></section>}
}
#[component]
fn Taxonomy(labels: bool) -> impl IntoView {
    let endpoint = if labels {
        "/api/v1/content/labels"
    } else {
        "/api/v1/content/categories"
    };
    let data = load(Signal::derive(move || {
        if labels {
            endpoint.into()
        } else {
            "/api/v1/categories".into()
        }
    }));
    let categories = load(Signal::derive(move || {
        if labels {
            "/api/v1/categories".into()
        } else {
            String::new()
        }
    }));
    let form = RwSignal::new(json!({"name":"","description":"","priority":0,"sort_id":0}));
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let deletion = Action::new();
    let deleting = RwSignal::new(None::<i64>);
    let reset = Callback::new(move |()| {
        editing.set(None);
        form.set(json!({"name":"","description":"","priority":0,"sort_id":0}));
    });
    view! {<section class="xcss-content-stack"><h1>{if labels{t("标签管理","Tags")}else{t("分类管理","Categories")}}</h1><Feedback error=data.error retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=categories.error fallback=t("分类加载失败","Unable to load categories") retry=Some(Callback::new(move|()|categories.reload()))/><Feedback error=save.error/><Feedback error=deletion.error/><fieldset class="xocs-fieldset" disabled=move||save.busy.get()><form class="inline-form" on:submit=move|ev|{ev.prevent_default();let current=form.get_untracked();if deletion.busy.get_untracked()&&deleting.get_untracked()==editing.get_untracked(){return;}
    if labels&&!list(categories.value.get_untracked()).iter().any(|c|num(c,"id")==num(&current,"sort_id")){return;}let mut body=json!({"name":current["name"],"description":if text(&current,"description").is_empty(){Value::Null}else{current["description"].clone()}});let order=if labels{"sort_id"}else{"priority"};body[order]=current[order].clone();let path=editing.get_untracked().map(|id|format!("{endpoint}/{id}")).unwrap_or(endpoint.into());save.run(path,if editing.get_untracked().is_some(){"PUT"}else{"POST"},Some(body),move |_|{reset.run(());data.reload();notify(if labels{t("标签已保存","Tag saved")}else{t("分类已保存","Category saved")});});}><Field label=if labels{t("标签名称","Tag name")}else{t("分类名称","Category name")} key="name" form required=true max=if labels{32}else{64}/><Field label=if labels{t("标签描述","Tag description")}else{t("分类描述","Category description")} key="description" form max=200/>{if labels{view!{<label class="xcss-form-field">{t("分类","Category")}<crate::select::Select label=Signal::derive(||t("分类","Category")) value=Signal::derive(move||num(&form.get(),"sort_id").to_string()) options=Signal::derive(move||std::iter::once(("0".to_string(),t("选择分类","Select a category"))).chain(list(categories.value.get()).into_iter().map(|c|(num(&c,"id").to_string(),text(&c,"sort_name")))).collect()) change=Callback::new(move|v:String|form.update(|f|f["sort_id"]=json!(v.parse::<i64>().unwrap_or(0)))) required=true disabled=Signal::derive(move||categories.value.get().is_none())/></label>}.into_any()}else{view!{<Field label=t("优先级","Priority") key="priority" form kind="number"/>}.into_any()}}<button class="xcss-button" type="submit" disabled=move||save.busy.get()||deletion.busy.get()&&deleting.get()==editing.get()>{move||if save.busy.get(){t("保存中…","Saving…")}else if editing.get().is_some(){t("保存修改","Save changes")}else if labels{t("新增标签","Add tag")}else{t("创建分类","Create category")}}</button>{move||editing.get().map(|_|view!{<button class="xcss-button" type="button" on:click=move |_|reset.run(())>{t("取消编辑","Cancel editing")}</button>})}</form>{move||data.value.get().or_else(||data.previous.get()).map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=if labels{t("标签列表","Tag list")}else{t("分类列表","Category list")}><thead><tr>{labels.then(||view!{<th>"ID"</th>})}<th>{if labels{t("标签","Tags")}else{t("名称","Name")}}</th><th>{t("描述","Description")}</th><th>{if labels{t("分类","Category")}else{t("排序","Order")}}</th><th>{t("操作","Actions")}</th></tr></thead><tbody>{v.as_array().cloned().unwrap_or_default().into_iter().map(|item|{let id=num(&item,"id");let edit=item.clone();view!{<tr>{labels.then(||view!{<td>{id}</td>})}<td>{text(&item,if labels{"label_name"}else{"sort_name"})}</td><td>{text(&item,if labels{"label_description"}else{"sort_description"})}</td><td>{if labels{list(categories.value.get()).iter().find(|c|num(c,"id")==num(&item,"sort_id")).map(|c|text(c,"sort_name")).unwrap_or_default()}else{num(&item,"priority").to_string()}}</td><td><div class="xcss-actions"><button class="xcss-button" on:click=move |_|{editing.set(Some(id));form.set(json!({"name":text(&edit,if labels{"label_name"}else{"sort_name"}),"description":text(&edit,if labels{"label_description"}else{"sort_description"}),"priority":num(&edit,"priority"),"sort_id":num(&edit,"sort_id")}));}>{t("编辑","Edit")}</button><button class="xcss-button xcss-danger" disabled=move||deletion.busy.get() on:click=move |_|{if save.busy.get_untracked()||!web_sys::window().unwrap().confirm_with_message(&if labels{t("确定删除这个标签？","Delete this tag?")}else{t("确定删除这个分类？","Delete this category?")}).unwrap_or(false){return;}deleting.set(Some(id));deletion.run(format!("{endpoint}/{id}"),"DELETE",None,move |_|{if editing.get_untracked()==Some(id){reset.run(())}for state in [data.value,data.previous]{state.update(|value|{if let Some(rows)=value.as_mut().and_then(Value::as_array_mut){rows.retain(|row|num(row,"id")!=id);}});}data.reload();notify(if labels{t("标签已删除","Tag deleted")}else{t("分类已删除","Category deleted")});});}>{t("删除","Delete")}</button></div></td></tr>}}).collect_view()}</tbody></table></div>})}</fieldset></section>}
}
fn moderation_author(item: &Value, message: bool) -> String {
    let username = text(item, "username");
    if !username.is_empty() {
        return username;
    }
    let user = num(item, "user_id");
    if user > 0 {
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
fn Moderation(kind: &'static str) -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
        format!("/api/v1/content/{kind}?page={}&size=20", page.get())
    }));
    let action = Action::new();
    let upload = Action::new();
    let lightbox = RwSignal::new(None::<String>);
    let chosen = StoredValue::new(send_wrapper::SendWrapper::new(None::<web_sys::File>));
    let filename = RwSignal::new(String::new());
    let uploaded = RwSignal::new(String::new());
    let title = match kind {
        "comments" => t("评论管理", "Comments"),
        "tree-hole" => t("留言板管理", "Message board"),
        _ => t("文件管理", "Files"),
    };
    view! {<section class="xcss-content-stack"><h1>{title}</h1><Feedback error=data.error retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=action.error/><Feedback error=upload.error/>{(kind=="resources").then(||view!{<fieldset class="xocs-fieldset" disabled=move||upload.busy.get()||action.busy.get()><form class="inline-form" on:submit=move|ev|{ev.prevent_default();let file=chosen.with_value(|f|f.as_ref().cloned());if let Some(file)=file{upload.upload(send_wrapper::SendWrapper::new(file),move|v|{uploaded.set(text(&v,"path"));filename.set(String::new());chosen.set_value(send_wrapper::SendWrapper::new(None));data.reload();notify(t("图片已上传","Image uploaded"));})}}><FilePicker label=t("上传图片（PNG、JPEG、GIF、WebP）","Upload image (PNG, JPEG, GIF, WebP)") filename=Signal::derive(move||filename.get()) disabled=Signal::derive(move||upload.busy.get()||action.busy.get()) choose=Callback::new(move|f:send_wrapper::SendWrapper<web_sys::File>|{filename.set(f.name());chosen.set_value(send_wrapper::SendWrapper::new(Some((*f).clone())));})/><button class="xcss-button" type="submit" disabled=move||filename.get().is_empty()||upload.busy.get()>{move||if upload.busy.get(){t("上传中…","Uploading…")}else{t("上传","Upload")}}</button></form></fieldset>{move||(!uploaded.get().is_empty()).then(||view!{<p>{t("新图片地址：","New image URL: ")}<code>{uploaded.get()}</code></p>})}})}{move||data.value.get().map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=match kind{"comments"=>t("评论列表","Comment list"),"tree-hole"=>t("留言列表","Message list"),_=>t("文件列表","File list")}><thead><tr>{(if kind=="comments"{vec![t("来源","Source"),t("用户","Users"),t("内容","Content"),t("时间","Time"),t("操作","Actions")]}else if kind=="tree-hole"{vec!["ID".into(),t("作者","Author"),t("内容","Content"),t("图片","Image"),t("时间","Time"),t("操作","Actions")]}else{vec![t("名称","Name"),t("类型","Type"),t("大小","Size"),t("位置","Location"),t("操作","Actions")]}).into_iter().map(|v|view!{<th>{v}</th>}).collect_view()}</tr></thead><tbody>{rows(&v,"items").into_iter().map(|item|{let id=num(&item,"id");let path=text(&item,"path");let path2=path.clone();view!{<tr>{if kind=="resources"{view!{<td>{text(&item,"original_name")}</td><td>{text(&item,"resource_type")}</td><td>{num(&item,"size")}</td><td>{safe_url(&path,false).map(|u|view!{<a href=u target="_blank" rel="noopener noreferrer">{t("打开","Open")}</a>})}</td>}.into_any()}else if kind=="comments"{view!{<td>{match text(&item,"comment_type").as_str(){"message"=>t("留言板","Message board").into_any(),"love"=>view!{<a href="/love" target="_blank" rel="noopener noreferrer">{t("祝福板","Wishes")}</a>}.into_any(),_=>view!{<a href=format!("/article/{}",num(&item,"source")) target="_blank" rel="noopener noreferrer">{format!("{}{}",t("文章 #","Article #"),num(&item,"source"))}</a>}.into_any()}}</td><td>{moderation_author(&item,false)}</td><td class="comment-cell">{text(&item,"comment_content")}</td><td>{table_time(&text(&item,"create_time"))}</td>}.into_any()}else{view!{<td>{id}</td><td>{moderation_author(&item,true)}</td><td>{let message=text(&item,"message");if message.is_empty(){t("图片留言","Image message")}else{message}}</td><td>{safe_url(&text(&item,"image_path"),true).map(|u|{let click=u.clone();view!{<button type="button" class="moderation-photo" on:click=move |_|lightbox.set(Some(click.clone()))><img src=u alt=t("查看留言图片","View message image")/></button>}})}</td><td>{table_time(&text(&item,"create_time"))}</td>}.into_any()}}<td><button class="xcss-button xcss-danger" disabled=move||action.busy.get()||upload.busy.get()||(kind=="resources"&&path2.is_empty()) on:click=move |_|{if !web_sys::window().unwrap().confirm_with_message(&match kind{"comments"=>t("删除这条评论？如果是顶层评论，会同时删除该楼层的全部回复。","Delete this comment? If it is a top-level comment, all replies in its thread will also be deleted."),"resources"=>t("确定删除这个文件及其图片数据？","Delete this file and its image data?"),_=>t("确定删除这条留言？","Delete this message?")}).unwrap_or(false){return;}let body=(kind=="resources").then(||json!({"path":path}));action.run(format!("/api/v1/content/{kind}/{id}"),"DELETE",body,move |_|data.reload());}>{t("删除","Delete")}</button></td></tr>}}).collect_view()}</tbody></table></div>})}<ListState data empty=match kind{"comments"=>t("暂无评论","No comments yet"),"tree-hole"=>t("暂无留言","No messages yet"),_=>t("暂无文件","No files yet")}/><Pager data page/><Lightbox src=lightbox/></section>}
}
#[component]
fn SiteSettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/site".into()));
    let form = RwSignal::new(Value::Null);
    let save = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            form.set(v);
        }
    });
    view! {<section class="xcss-content-stack"><h1>{t("网站设置","Site settings")}</h1><Feedback error=data.error retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=save.error/>{move||(!form.get().is_null()).then(||view!{<form class="xocs-settings-form" on:submit=move|ev|{ev.prevent_default();let mut f=form.get_untracked();let name=text(&f,"author_name").trim().to_string();let len=name.chars().count();if !(1..=100).contains(&len){return;}f["author_name"]=json!(name);save.run("/api/v1/content/site".into(),"PUT",Some(f),move|v|{form.set(v);notify(t("站点设置已保存","Site settings saved"));});}><fieldset class="xocs-fieldset" disabled=move||save.busy.get() aria-label=t("网站设置","Site settings")><section class="xcss-content-panel"><h2>{t("基本信息","Basic information")}</h2><div class="form-grid"><Field label=t("网站名称","Site name") key="web_name" form required=true/><Field label=t("网站标题","Site title") key="web_title" form/><Field label=t("站长显示名","Site owner display name") key="author_name" form required=true max=200/></div><p class="xocs-field-help">{t("用于文章和随笔署名；管理员登录名与密码在右上角头像的账号设置中修改。","Used as the author of articles and notes. Change administrator sign-in credentials through the account settings avatar.")}</p><Field label=t("公告","Notices") key="notices" form textarea=true/><p class="xocs-field-help">{t("使用 JSON 字符串数组或每行一条公告。","Use a JSON string array or one notice per line.")}</p><details><summary>{t("推送格式","Promotion format")}</summary><p><code>"推送标题："</code>" "<code>"推送封面："</code>" "<code>"推送链接："</code></p></details></section><section class="xcss-content-panel"><h2>{t("外观","Appearance")}</h2><div class="form-grid"><Field label=t("背景图片","Background image") key="background_image" form/><Field label=t("站长头像","Site owner avatar") key="avatar" form/></div></section><section class="xcss-content-panel"><h2>{t("随机封面","Random covers")}</h2><p class="xocs-field-help">{t("使用 JSON 图片地址数组或每行一个图片地址。","Use a JSON array of image URLs or one image URL per line.")}</p><Field label=t("随机封面","Random covers") key="random_cover" form textarea=true/></section></fieldset><div class="xcss-actions"><button class="xcss-button" type="submit" disabled=move||save.busy.get()>{t("保存设置","Save settings")}</button></div></form>})}</section>}
}
#[component]
fn LetterSettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/letter".into()));
    let form = RwSignal::new(Value::Null);
    let original = RwSignal::new(Value::Null);
    let save = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            form.set(v.clone());
            original.set(v);
        }
    });
    view! {<section class="xcss-content-stack"><h1>{t("信笺管理","Letter")}</h1><p>{t("维护公开信笺的收信人、署名和正文。","Manage the recipient, sender, and content of the public letter.")}</p><Feedback error=data.error retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=save.error/>{move||(!form.get().is_null()).then(||view!{<form class="editor-form" on:submit=move|ev|{ev.prevent_default();let f=form.get_untracked();let valid=[("recipient",100), ("sender",100),("content",16384)].iter().all(|(k,max)|{let n=text(&f,k).trim().chars().count();n>=1&&n<=*max});if valid{save.run("/api/v1/content/letter".into(),"PUT",Some(f),move|v|{form.set(v.clone());original.set(v);notify(t("信笺已保存","Letter saved"));});}}><fieldset class="xocs-fieldset" disabled=move||save.busy.get()><Field label=t("收信人","Recipient") key="recipient" form required=true max=200/><Field label=t("署名","Sender") key="sender" form required=true max=200/><Field label=t("信笺正文","Letter content") key="content" form required=true textarea=true max=32768/><Check label=t(" 公开显示"," Show publicly") key="is_public" form/><div class="xcss-actions"><button class="xcss-button" type="submit" disabled=move||save.busy.get()>{t("保存信笺","Save letter")}</button><button class="xcss-button" type="button" disabled=move||save.busy.get()||form.get()==original.get() on:click=move |_|{form.set(original.get_untracked());save.error.set(None);} >{t("撤销修改","Reset changes")}</button><a href=href("/letter") target="_blank" rel="noopener noreferrer">{t("查看公开页面","View public page")}</a></div></fieldset></form>})}</section>}
}
fn empty_family() -> Value {
    json!({"bg_cover":"","man_cover":"","woman_cover":"","man_name":"","woman_name":"","timing":"","countdown_title":"","countdown_time":"","family_info":"","status":true})
}
fn family_form(item: &Value) -> Value {
    let mut form = empty_family();
    for key in [
        "bg_cover",
        "man_cover",
        "woman_cover",
        "man_name",
        "woman_name",
        "timing",
        "countdown_title",
        "countdown_time",
        "family_info",
    ] {
        form[key] = json!(text(item, key));
    }
    form["status"] = json!(flag(item, "status"));
    form
}
#[component]
fn FamilySettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/family".into()));
    let form = RwSignal::new(empty_family());
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            if let Some(item) = v.as_array().and_then(|a| a.first()) {
                editing.set(Some(num(item, "id")));
                form.set(family_form(item));
            } else {
                editing.set(None);
                form.set(empty_family());
            }
        }
    });
    view! {<section class="xcss-content-stack"><h1>{t("恋爱笔记","Love journal")}</h1><p>{t("这里仅由管理员创建并维护一份家的资料。","The administrator creates and maintains one home here.")}</p><Feedback error=data.error retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=save.error/><fieldset class="xocs-fieldset" disabled=move||save.busy.get()||data.value.get().is_none()><form class="editor-form" on:submit=move|ev|{ev.prevent_default();if data.value.get_untracked().is_none(){return;}let mut payload=form.get_untracked();if let Some(o)=payload.as_object_mut(){o.remove("id");}for k in ["bg_cover","man_cover","woman_cover","countdown_title","countdown_time","family_info"]{if text(&payload,k).is_empty(){payload[k]=Value::Null;}}let path=editing.get_untracked().map(|id|format!("/api/v1/content/family/{id}")).unwrap_or("/api/v1/content/family".into());save.run(path,if editing.get_untracked().is_some(){"PUT"}else{"POST"},Some(payload),move|v|{editing.set(Some(num(&v,"id")));form.set(family_form(&v));notify(t("家的资料已保存","Home details saved"));});}><h2>{move||if editing.get().is_some(){t("编辑家的资料","Edit home details")}else{t("创建我的家","Create my home")}}</h2><div class="form-grid"><Field label=t("男生昵称","His name") key="man_name" form required=true max=32/><Field label=t("女生昵称","Her name") key="woman_name" form required=true max=32/></div><div class="form-grid"><Field label=t("起始时间","Start date") key="timing" form max=32/><Field label=t("倒计时标题","Countdown title") key="countdown_title" form max=32/></div><Field label=t("倒计时时间","Countdown date") key="countdown_time" form max=32/><Field label=t("背景图片","Background image") key="bg_cover" form/><div class="form-grid"><Field label=t("男生头像","His avatar") key="man_cover" form/><Field label=t("女生头像","Her avatar") key="woman_cover" form/></div><Field label=t("简介","Introduction") key="family_info" form textarea=true max=1024/><Check label=t(" 公开显示"," Show publicly") key="status" form/><div class="xcss-actions"><button class="xcss-button" type="submit">{t("保存","Save")}</button>{move||editing.get().map(|id|view!{<button class="xcss-button xcss-danger" type="button" on:click=move |_|{if web_sys::window().unwrap().confirm_with_message(&t("确定删除家的资料？","Delete your home details?")).unwrap_or(false){save.run(format!("/api/v1/content/family/{id}"),"DELETE",None,move |_|data.reload());}}>{t("删除家的资料","Delete home details")}</button>})}</div></form></fieldset></section>}
}
#[component]
fn HomeSections() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/home-sections".into()));
    let categories = load(Signal::derive(|| "/api/v1/categories".into()));
    let sections = RwSignal::new(Vec::<Value>::new());
    let loaded = RwSignal::new(false);
    let save = Action::new();
    let validation = RwSignal::new(String::new());
    Effect::new(move |_| {
        if let Some(v) = data.value.get()
            && !loaded.get_untracked()
        {
            sections.set(v.as_array().cloned().unwrap_or_default());
            loaded.set(true);
        }
    });
    let save_sections = Callback::new(move |()| {
        if !loaded.get_untracked() || save.busy.get_untracked() {
            return;
        }
        let payload:Vec<Value>=sections.get_untracked().into_iter().enumerate().map(|(i,v)|json!({"title":text(&v,"title").trim(),"kind":v["kind"],"sort_id":if v["kind"]=="category"{v["sort_id"].clone()}else{Value::Null},"priority":i*10,"enabled":flag(&v,"enabled")})).collect();
        if payload.iter().any(|v| {
            text(v, "title").is_empty() || v["kind"] == "category" && num(v, "sort_id") == 0
        }) {
            validation.set(t(
                "请填写栏目标题并选择分类",
                "Enter a section title and select a category",
            ));
            return;
        }
        validation.set(String::new());
        save.run(
            "/api/v1/content/home-sections".into(),
            "PUT",
            Some(json!(payload)),
            move |v| {
                sections.set(v.as_array().cloned().unwrap_or_default());
                notify(t("首页栏目已保存", "Home sections saved"));
            },
        )
    });
    view! {<section class="xcss-content-stack"><h1>{t("首页栏目","Home sections")}</h1><p>{t("设置栏目、顺序和展示内容。推荐栏目使用文章的“推荐”状态。","Set sections, their order, and content. Recommended sections use articles marked as recommended.")}</p><button class="xcss-button" disabled=move||save.busy.get()||!loaded.get() on:click=move |_|{save_sections.run(())}>{t("保存栏目","Save sections")}</button><Feedback error=data.error fallback=t("首页栏目加载失败","Unable to load home sections") retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=categories.error fallback=t("分类加载失败","Unable to load categories") retry=Some(Callback::new(move|()|categories.reload()))/><Feedback error=save.error fallback=t("保存失败","Unable to save") retry=Some(save_sections)/>{move||(!validation.get().is_empty()).then(||view!{<p role="alert">{validation.get()}</p>})}<fieldset class="xocs-fieldset" disabled=move||save.busy.get()||!loaded.get()><div class="xcss-actions"><button class="xcss-button" disabled={move||sections.get().len()>=12} on:click=move |_|sections.update(|rows|rows.push(json!({"id":-(js_sys::Date::now() as i64),"title":t("新栏目","New section"),"kind":"category","sort_id":list(categories.value.get_untracked()).first().map(|v|v["id"].clone()).unwrap_or(Value::Null),"priority":rows.len(),"enabled":true})))>{t("新增分类栏目","Add category section")}</button></div><div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=t("首页栏目列表","Home section list")><thead><tr>{[t("顺序","Order"),t("标题","Title"),t("内容来源","Content source"),t("分类","Category"),t("显示","Show"),t("操作","Actions")].into_iter().map(|l|view!{<th>{l}</th>}).collect_view()}</tr></thead><tbody><For each=move||sections.get() key=|v|num(v,"id") children=move|item|view!{<HomeRow id=num(&item,"id") sections categories/>}/></tbody></table></div></fieldset></section>}
}
#[component]
fn HomeRow(id: i64, sections: RwSignal<Vec<Value>>, categories: Data) -> impl IntoView {
    let index = Signal::derive(move || {
        sections
            .get()
            .iter()
            .position(|v| num(v, "id") == id)
            .unwrap_or(0)
    });
    let current = Signal::derive(move || {
        sections
            .get()
            .into_iter()
            .find(|v| num(v, "id") == id)
            .unwrap_or(Value::Null)
    });
    let change = Callback::new(move |(k, v): (&'static str, Value)| {
        sections.update(|rows| {
            if let Some(row) = rows.iter_mut().find(|v| num(v, "id") == id) {
                row[k] = v
            }
        })
    });
    view! {<tr><td>{move||index.get()+1}</td><td><label class="xcss-form-field">{move||format!("{} {} {}",t("栏目","Section"),index.get()+1,t("标题","title"))}<input class="xcss-input" maxlength="32" prop:value=move||text(&current.get(),"title") on:input=move|ev|change.run(("title",json!(event_target_value(&ev))))/></label></td><td><crate::select::Select label=Signal::derive(move||tr("栏目 {0} 内容来源","Section {0} content source",&[(index.get()+1).to_string()])) value=Signal::derive(move||text(&current.get(),"kind")) options=Signal::derive(||vec![("latest".into(),t("最新文章","Latest articles")),("recommended".into(),t("推荐文章","Recommended articles")),("category".into(),t("指定分类","Selected category"))]) change=Callback::new(move|kind:String|{change.run(("sort_id",if kind=="category"{list(categories.value.get_untracked()).first().map(|v|v["id"].clone()).unwrap_or(Value::Null)}else{Value::Null}));change.run(("kind",json!(kind)));})/></td><td>{move||(text(&current.get(),"kind")=="category").then(||view!{<crate::select::Select label=Signal::derive(move||tr("栏目 {0} 分类","Section {0} category",&[(index.get()+1).to_string()])) value=Signal::derive(move||num(&current.get(),"sort_id").to_string()) options=Signal::derive(move||std::iter::once(("0".to_string(),t("选择分类","Select a category"))).chain(list(categories.value.get()).into_iter().map(|c|(num(&c,"id").to_string(),text(&c,"sort_name")))).collect()) change=Callback::new(move|v:String|change.run(("sort_id",json!(v.parse::<i64>().unwrap_or(0)))))/>})}</td><td><label><input type="checkbox" prop:checked=move||flag(&current.get(),"enabled") on:change=move|ev|change.run(("enabled",json!(event_target_checked(&ev))))/>{t(" 启用"," Enable")}</label></td><td><div class="xcss-actions"><button class="xcss-button" aria-label=move||format!("{}{}",t("上移栏目 ","Move section up "),index.get()+1) disabled=move||index.get()==0 on:click=move |_|{let i=index.get_untracked();sections.update(|r|if i>0{r.swap(i,i-1)})}>"↑"</button><button class="xcss-button" aria-label=move||format!("{}{}",t("下移栏目 ","Move section down "),index.get()+1) disabled={move||index.get()+1>=sections.get().len()} on:click=move |_|{let i=index.get_untracked();sections.update(|r|if i+1<r.len(){r.swap(i,i+1)})}>"↓"</button><button class="xcss-button xcss-danger" on:click=move |_|sections.update(|r|r.retain(|v|num(v,"id")!=id))>{t("删除","Delete")}</button></div></td></tr>}
}
fn empty_link() -> Value {
    json!({"title":"","classify":"","cover":"","url":"","introduction":"","link_type":"favorites","status":true})
}
#[component]
fn Links() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let kind = RwSignal::new(String::new());
    let status = RwSignal::new(-1i64);
    let data = load(Signal::derive(move || {
        format!(
            "/api/v1/content/links?page={}&size=20{}{}",
            page.get(),
            if kind.get().is_empty() {
                String::new()
            } else {
                format!("&kind={}", enc(&kind.get()))
            },
            if status.get() < 0 {
                String::new()
            } else {
                format!("&status={}", status.get())
            }
        )
    }));
    let form = RwSignal::new(empty_link());
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let remove = Action::new();
    let reset = Callback::new(move |()| {
        form.set(empty_link());
        editing.set(None)
    });
    let options = || {
        vec![
            ("favorites", t("收藏", "Favorites")),
            ("friendUrl", t("友链", "Friend links")),
            ("lovePhoto", t("照片", "Photos")),
            ("funny", t("音乐与趣味", "Music and fun")),
        ]
    };
    view! {<section class="xcss-content-stack"><h1>{t("内容资源","Content resources")}</h1><p>{t("维护友链、收藏、照片和音乐条目。","Manage friend links, favorites, photos, and music.")}</p><Feedback error=data.error retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=save.error/><Feedback error=remove.error/><div class="inline-form"><label class="xcss-form-field">{t("类型筛选","Filter by type")}<crate::select::Select label=Signal::derive(||t("类型筛选","Filter by type")) value=Signal::derive(move||kind.get()) options=Signal::derive(move||std::iter::once((String::new(),t("全部","All"))).chain(options().into_iter().map(|(k,l)|(k.to_string(),l))).collect()) change=Callback::new(move|v:String|{kind.set(v);page.set(1);})/></label><label class="xcss-form-field">{t("状态筛选","Filter by status")}<crate::select::Select label=Signal::derive(||t("状态筛选","Filter by status")) value=Signal::derive(move||status.get().to_string()) options=Signal::derive(||vec![("-1".into(),t("全部","All")),("0".into(),t("待审核/隐藏","Pending review / hidden")),("1".into(),t("已公开","Published"))]) change=Callback::new(move|v:String|{status.set(v.parse().unwrap_or(-1));page.set(1);})/></label></div><fieldset class="xocs-fieldset" disabled=move||save.busy.get()><form class="editor-form" on:submit=move|ev|{ev.prevent_default();let mut f=form.get_untracked();for k in ["classify","cover","url","introduction"]{if text(&f,k).is_empty(){f[k]=Value::Null;}}let path=editing.get_untracked().map(|id|format!("/api/v1/content/links/{id}")).unwrap_or("/api/v1/content/links".into());save.run(path,if editing.get_untracked().is_some(){"PUT"}else{"POST"},Some(f),move |_|{reset.run(());data.reload();notify(t("资源已保存","Resource saved"));});}><h2>{move||if editing.get().is_some(){t("编辑资源","Edit resource")}else{t("新增资源","Add resource")}}</h2><div class="form-grid"><label class="xcss-form-field">{t("类型","Type")}<crate::select::Select label=Signal::derive(||t("类型","Type")) value=Signal::derive(move||text(&form.get(),"link_type")) options=Signal::derive(move||options().into_iter().map(|(k,l)|(k.to_string(),l)).collect()) change=Callback::new(move|v:String|form.update(|f|f["link_type"]=json!(v)))/></label><Field label=t("分类","Category") key="classify" form max=32/></div><Field label=t("标题","Title") key="title" form required=true max=64/><Field label=t("图片地址","Image URL") key="cover" form/><Field label=t("目标地址","Target URL") key="url" form/><Field label=t("简介","Introduction") key="introduction" form/><Check label=t(" 公开显示"," Show publicly") key="status" form/><div class="xcss-actions"><button class="xcss-button" type="submit">{move||if editing.get().is_some(){t("保存修改","Save changes")}else{t("新增资源","Add resource")}}</button>{move||editing.get().map(|_|view!{<button class="xcss-button" type="button" on:click=move |_|reset.run(())>{t("取消编辑","Cancel editing")}</button>})}</div></form>{move||data.value.get().map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=t("内容资源列表","Content resource list")><thead><tr><th>{t("标题","Title")}</th><th>{t("类型","Type")}</th><th>{t("状态","Status")}</th><th>{t("操作","Actions")}</th></tr></thead><tbody>{rows(&v,"items").into_iter().map(|item|{let id=num(&item,"id");let edit=item.clone();view!{<tr><td>{text(&item,"title")}</td><td>{text(&item,"link_type")}</td><td>{if flag(&item,"status"){t("公开","Public")}else{t("待审核/隐藏","Pending review / hidden")}}</td><td><div class="xcss-actions"><button class="xcss-button" on:click=move |_|{editing.set(Some(id));let mut f=empty_link();for k in ["title","classify","cover","url","introduction","link_type"]{f[k]=json!(text(&edit,k));}f["status"]=json!(flag(&edit,"status"));form.set(f);}>{t("编辑","Edit")}</button>{(text(&item,"link_type")=="friendUrl"&&!flag(&item,"status")).then(||view!{<button class="xcss-button" on:click=move |_|remove.run(format!("/api/v1/content/links/{id}/status"),"PUT",Some(json!({"active":true})),move |_|data.reload())>{t("审核通过","Approve")}</button>})}<button class="xcss-button xcss-danger" on:click=move |_|{if web_sys::window().unwrap().confirm_with_message(&t("确定删除这条资源？","Delete this resource?")).unwrap_or(false){remove.run(format!("/api/v1/content/links/{id}"),"DELETE",None,move |_|data.reload());}}>{t("删除","Delete")}</button></div></td></tr>}}).collect_view()}</tbody></table></div>})}<ListState data empty=t("暂无内容资源","No content resources yet")/><Pager data page/></fieldset></section>}
}
