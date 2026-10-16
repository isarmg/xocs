use crate::common::Action;
use crate::common::*;
use leptos::prelude::*;
use serde_json::{Value, json};
use xcss_web::components::notify;
fn empty_article() -> Value {
    json!({"article_title":"","article_content":"","article_cover":null,"video_url":null,"sort_id":0,"label_id":0,"view_status":true,"recommend_status":false,"comment_status":true,"password":null,"tips":null})
}
#[component]
pub fn ArticleEditor(#[prop(default = 0)] id: i64) -> impl IntoView {
    let categories = load(Signal::derive(|| "/api/v1/categories".into()));
    let labels = load(Signal::derive(|| "/api/v1/content/labels".into()));
    let article = load(Signal::derive(move || {
        if id > 0 {
            format!("/api/v1/content/articles/{id}")
        } else {
            String::new()
        }
    }));
    let form = RwSignal::new(empty_article());
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
            let mut f = empty_article();
            for k in [
                "article_title",
                "article_content",
                "article_cover",
                "video_url",
                "sort_id",
                "label_id",
                "tips",
            ] {
                f[k] = v[k].clone();
            }
            for k in ["view_status", "recommend_status", "comment_status"] {
                f[k] = json!(flag(&v, k));
            }
            form.set(f);
            existing.set(flag(&v, "password_required"));
            ready.set(true);
        }
    });
    let upload_file = move |file: send_wrapper::SendWrapper<web_sys::File>, cover: bool| {
        if !ready.get_untracked() || save.busy.get_untracked() {
            return;
        }
        let name = file.name().replace(['[', ']'], "");
        upload.upload(file, move |v| {
            let path = text(&v, "path");
            form.update(|f| {
                if cover {
                    f["article_cover"] = json!(path)
                } else {
                    f["article_content"] = json!(format!(
                        "{}\n\n![{name}]({path})\n",
                        text(f, "article_content")
                    ))
                }
            });
            notify(t("图片已上传", "Image uploaded"));
        });
    };
    view! {<section class="xcss-content-stack"><h1>{if id>0{t("编辑文章","Edit article")}else{t("新增文章","Add article")}}</h1><p>{t("支持 Markdown 正文，发布后可从网站访问。","Write in Markdown. Published articles are available on the site.")}</p><Feedback error=categories.error fallback=t("分类加载失败","Unable to load categories") retry=Some(Callback::new(move|()|categories.reload()))/><Feedback error=labels.error fallback=t("标签加载失败","Unable to load tags") retry=Some(Callback::new(move|()|labels.reload()))/><Feedback error=article.error fallback=t("文章加载失败","Unable to load article") retry=Some(Callback::new(move|()|article.reload()))/><Feedback error=save.error fallback=t("尚未确认文章是否已创建，请保留原内容重试。","Article creation is not yet confirmed. Retry with the original content unchanged.")/><Feedback error=upload.error/>{move||(!validation.get().is_empty()).then(||view!{<p role="alert">{validation.get()}</p>})}<Show when=move||ready.get() fallback=move||view!{<p role="status">{t("正在加载…","Loading…")}</p><button class="xcss-button" on:click=move |_|navigate("articles")>{t("取消","Cancel")}</button>}><form class="editor-form" on:submit=move|ev|{ev.prevent_default();if upload.busy.get_untracked()||!ready.get_untracked(){return;}let mut f=form.get_untracked();let issue=if flag(&f,"view_status"){None}else{crate::dates::password_error(&text(&f,"password"),existing.get_untracked())};if let Some(issue)=issue{validation.set(issue);return;}for k in ["article_cover","video_url","password","tips"]{if text(&f,k).is_empty(){f[k]=Value::Null;}}let target=if id>0{Some(id)}else{created_id.get_value()};let path=target.map(|id|format!("/api/v1/content/articles/{id}")).unwrap_or("/api/v1/content/articles".into());let opts=RequestOptions{headers:if target.is_none(){vec![("Idempotency-Key".into(),creation.get_value())]}else{vec![]},..Default::default()};save.run_options(path,if target.is_some(){"PUT"}else{"POST"},Some(f),opts,move |v|{if id==0{created_id.set_value(Some(num(&v,"id")));}notify(t("文章已保存","Article saved"));navigate("articles");});}><fieldset class="xocs-fieldset" disabled=move||save.busy.get()><Field label=t("标题","Title") key="article_title" form required=true max=120/><div class="form-grid"><label class="xcss-form-field">{t("分类","Category")}<crate::select::Select label=Signal::derive(||t("分类","Category")) value=Signal::derive(move||num(&form.get(),"sort_id").to_string()) options=Signal::derive(move||std::iter::once(("0".to_string(),t("选择分类","Select a category"))).chain(list(categories.value.get()).into_iter().map(|c|(num(&c,"id").to_string(),text(&c,"sort_name")))).collect()) required=true change=Callback::new(move|v:String|form.update(|f|{f["sort_id"]=json!(v.parse::<i64>().unwrap_or(0));f["label_id"]=json!(0);}))/></label><label class="xcss-form-field">{t("标签","Tags")}<crate::select::Select label=Signal::derive(||t("标签","Tags")) value=Signal::derive(move||num(&form.get(),"label_id").to_string()) options=Signal::derive(move||std::iter::once(("0".to_string(),t("选择标签","Select a tag"))).chain(list(labels.value.get()).into_iter().filter(|c|num(c,"sort_id")==num(&form.get(),"sort_id")).map(|c|(num(&c,"id").to_string(),text(&c,"label_name")))).collect()) required=true change=Callback::new(move|v:String|form.update(|f|f["label_id"]=json!(v.parse::<i64>().unwrap_or(0))))/></label></div><div class="form-grid"><Field label=t("封面图片地址","Cover image URL") key="article_cover" form/><FilePicker label=t("上传封面图片","Upload cover image") disabled=Signal::derive(move||save.busy.get()||upload.busy.get()) choose=Callback::new(move|file|upload_file(file,true))/></div><label class="xcss-form-field">{t("Markdown 正文","Markdown content")}<textarea class="xcss-input editor-textarea" required prop:value=move||text(&form.get(),"article_content") on:input=move|ev|form.update(|f|f["article_content"]=json!(event_target_value(&ev)))/></label><div class="xcss-actions"><button class="xcss-button" type="button" on:click=move |_|preview.update(|v|*v= !*v)>{move||if preview.get(){t("收起预览","Hide preview")}else{t("预览正文","Preview content")}}</button><FilePicker label=t("插入正文图片","Insert content image") disabled=Signal::derive(move||save.busy.get()||upload.busy.get()) choose=Callback::new(move|file|upload_file(file,false))/></div>{move||preview.get().then(||view!{<div class="editor-markdown-preview article-markdown" inner_html=crate::browser::markdown_html(&text(&form.get(),"article_content"))/>})}<div class="form-grid"><Field label=t("视频地址","Video URL") key="video_url" form/>{move||(!flag(&form.get(),"view_status")).then(||view!{<div><label class="xcss-form-field">{t("文章访问密码","Article access password")}<input class="xcss-input" type="password" aria-describedby=move||crate::dates::password_error(&text(&form.get(),"password"),existing.get()).map(|_|"xocs-article-password-error") aria-invalid=move||crate::dates::password_error(&text(&form.get(),"password"),existing.get()).is_some().to_string() placeholder=move||if existing.get(){t("留空则保留原密码","Leave empty to keep the current password")}else{t("至少 12 字节","At least 12 bytes")} prop:value=move||text(&form.get(),"password") on:input=move|ev|form.update(|f|f["password"]=json!(event_target_value(&ev)))/></label>{move||crate::dates::password_error(&text(&form.get(),"password"),existing.get()).map(|e|view!{<p id="xocs-article-password-error" role="alert">{e}</p>})}</div>})}</div><div class="check-row"><label><input type="checkbox" prop:checked=move||flag(&form.get(),"view_status") on:change=move|ev|{let v=event_target_checked(&ev);form.update(|f|{f["view_status"]=json!(v);if v{f["password"]=Value::Null;}});}/>{t(" 公开访问（关闭后需设置密码）"," Public access (a password is required when disabled)")}</label><Check label=t(" 推荐"," Recommended") key="recommend_status" form/><Check label=t(" 允许评论"," Allow comments") key="comment_status" form/></div>{move||(!flag(&form.get(),"view_status")).then(||view!{<Field label=t("密码提示","Password hint") key="tips" form/><span class="xocs-field-help">{t("显示在文章访问密码输入页。","Displayed on the article access password screen.")}</span>})}<div class="xcss-actions"><button class="xcss-button" type="button" disabled=move||save.busy.get() on:click=move |_|{upload.abort();navigate("articles")}>{t("取消","Cancel")}</button><button class="xcss-button" type="submit" disabled=move||save.busy.get()||upload.busy.get()>{move||if upload.busy.get(){t("图片上传中…","Uploading image…")}else if save.busy.get(){t("保存中…","Saving…")}else{t("保存文章","Save article")}}</button></div></fieldset></form>{(id>0).then(||view!{<NewsEditor id/>})}</Show></section>}
}
#[component]
fn NewsEditor(id: i64) -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
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
    view! {<section class="editor-form"><h2>{t("最新进展","Recent updates")}</h2><Feedback error=data.error fallback=t("文章进展加载失败","Unable to load article updates") retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=save.error/>{move||(!validation.get().is_empty()).then(||view!{<p role="alert">{validation.get()}</p>})}<form class="editor-form" on:submit=move|ev|{ev.prevent_default();let f=form.get_untracked();let content=text(&f,"content");if !(1..=1024).contains(&content.trim().chars().count()){validation.set(t("进展正文须为 1–1024 个字符","Update content must contain 1–1024 characters"));return;}let input=json!({"content":content,"create_time":crate::dates::iso(&text(&f,"create_time"))});let body=input.to_string();if editing.get_untracked().is_none()&&receipt.get_value().as_ref().is_none_or(|(_,b)|*b!=body){receipt.set_value(Some((web_sys::window().unwrap().crypto().unwrap().random_uuid(),body)));}let target=editing.get_untracked();let path=format!("/api/v1/content/articles/{id}/news{}",target.map(|id|format!("/{id}")).unwrap_or_default());let opts=RequestOptions{headers:if target.is_none(){receipt.get_value().map(|(k,_)|vec![("Idempotency-Key".into(),k)]).unwrap_or_default()}else{vec![]},..Default::default()};save.run_options(path,if target.is_some(){"PUT"}else{"POST"},Some(input),opts,move|v|{if text(&v,"content")!=content.trim()||target.is_some_and(|i|num(&v,"id")!=i){save.error.set(Some(invalid()));return;}reset.run(());page.set(1);data.reload();notify(t("文章进展已保存","Article update saved"));});}><fieldset class="xocs-fieldset" disabled=move||save.busy.get()||data.value.get().is_none()><h3>{move||if editing.get().is_some(){t("编辑进展","Edit update")}else{t("新增进展","Add update")}}</h3><Field label=t("进展内容","Update content") key="content" form textarea=true required=true max=2048/><label class="xcss-form-field">{t("进展时间（UTC）","Update time (UTC)")}<input class="xcss-input" type="datetime-local" step="1" prop:value=move||text(&form.get(),"create_time") on:input=move|ev|form.update(|f|f["create_time"]=json!(event_target_value(&ev)))/></label><p class="xocs-field-help">{t("新增时留空使用当前 UTC；编辑时留空保留原时间。","Leave blank to use current UTC for a new update, or keep its existing time when editing.")}</p><div class="xcss-actions"><button class="xcss-button" type="submit">{move||if editing.get().is_some(){t("保存修改","Save changes")}else{t("发布进展","Publish update")}}</button>{move||(editing.get().is_some()||form.get()!=original.get()).then(||view!{<button class="xcss-button" type="button" on:click=move |_|{if form.get_untracked()==original.get_untracked()||web_sys::window().unwrap().confirm_with_message(&t("放弃当前未保存的进展修改？","Discard the current unsaved update changes?")).unwrap_or(false){reset.run(())}}>{t("取消编辑","Cancel editing")}</button>})}</div></fieldset></form>{move||data.value.get().map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=t("文章进展","Article updates")><thead><tr><th>{t("时间","Time")}</th><th>{t("内容","Content")}</th><th>{t("操作","Actions")}</th></tr></thead><tbody>{rows(&v,"items").into_iter().map(|item|{let entry_id=num(&item,"id");let edit=item.clone();view!{<tr><td>{crate::dates::format(&text(&item,"create_time"))}</td><td class="comment-cell">{text(&item,"content")}</td><td><div class="xcss-actions"><button class="xcss-button" disabled=move||save.busy.get() on:click=move |_|{if form.get_untracked()!=original.get_untracked()&&!web_sys::window().unwrap().confirm_with_message(&t("放弃当前未保存的进展修改？","Discard the current unsaved update changes?")).unwrap_or(false){return;}let f=json!({"content":edit["content"],"create_time":crate::dates::input(&text(&edit,"create_time"))});form.set(f.clone());original.set(f);editing.set(Some(entry_id));}>{t("编辑","Edit")}</button><button class="xcss-button xcss-danger" disabled=move||save.busy.get() on:click=move |_|{if web_sys::window().unwrap().confirm_with_message(&t("确定删除这条文章进展？","Delete this article update?")).unwrap_or(false){save.run(format!("/api/v1/content/articles/{id}/news/{entry_id}"),"DELETE",None,move |_|{if editing.get_untracked()==Some(entry_id){reset.run(())}data.reload();});}}>{t("删除","Delete")}</button></div></td></tr>}}).collect_view()}</tbody></table></div>})}<Pager data page label=t("进展分页","Update pages")/></section>}
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
    view! {<section class="xcss-content-stack"><h1>{t("随笔管理","Notes")}</h1><p>{t("维护随笔及恋爱日常中的内容。","Manage notes shown in the journal and everyday moments.")}</p><Feedback error=data.error fallback=t("随笔加载失败","Unable to load notes") retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=save.error/><Feedback error=upload.error/>{move||(!validation.get().is_empty()).then(||view!{<p role="alert">{validation.get()}</p>})}<form class="editor-form" on:submit=move|ev|{ev.prevent_default();if upload.busy.get_untracked(){return;}let f=form.get_untracked();let content=text(&f,"content");if !(1..=1024).contains(&content.trim().chars().count()){validation.set(t("随笔正文须为 1–1024 个字符","Note content must contain 1–1024 characters"));return;}let id=editing.get_untracked();let expected=f.clone();let path=id.map(|id|format!("/api/v1/content/notes/{id}")).unwrap_or("/api/v1/content/notes".into());save.run(path,if id.is_some(){"PUT"}else{"POST"},Some(f),move|v|{if text(&v,"content")!=content.trim()||v["image_path"]!=expected["image_path"]||flag(&v,"is_public")!=flag(&expected,"is_public")||id.is_some_and(|id|num(&v,"id")!=id){save.error.set(Some(invalid()));return;}reset.run(());page.set(1);data.reload();notify(t("随笔已保存","Note saved"));});}><fieldset class="xocs-fieldset" disabled=move||save.busy.get()><h2>{move||if editing.get().is_some(){t("编辑随笔","Edit note")}else{t("新增随笔","Add note")}}</h2><Field label=t("随笔正文","Note content") key="content" form textarea=true required=true max=2048/><FilePicker label=t("随笔图片","Note image") filename=Signal::derive(move||filename.get()) disabled=Signal::derive(move||upload.busy.get()||save.busy.get()) choose=Callback::new(move|file:send_wrapper::SendWrapper<web_sys::File>|{let name=file.name();upload.upload(file,move|v|{form.update(|f|f["image_path"]=v["path"].clone());filename.set(name);notify(t("图片已上传","Image uploaded"));});})/>{move||safe_url(&text(&form.get(),"image_path"),true).map(|url|{let click=url.clone();view!{<div><button type="button" class="moderation-photo" on:click=move |_|lightbox.set(Some(click.clone()))><img src=url alt=t("查看随笔图片","View note image")/></button><button type="button" class="xcss-button" disabled=move||upload.busy.get()||save.busy.get() on:click=move |_|{form.update(|f|f["image_path"]=Value::Null);filename.set(String::new())}>{t("移除图片","Remove image")}</button></div>}})}<Check label=t(" 公开可见"," Publicly visible") key="is_public" form/><div class="xcss-actions"><button class="xcss-button" type="submit" disabled=move||save.busy.get()||upload.busy.get()>{move||if upload.busy.get(){t("图片上传中…","Uploading image…")}else if save.busy.get(){t("保存中…","Saving…")}else if editing.get().is_some(){t("保存修改","Save changes")}else{t("发布","Publish")}}</button>{move||(editing.get().is_some()||form.get()!=original.get()||upload.busy.get()).then(||view!{<button class="xcss-button" type="button" disabled=move||save.busy.get() on:click=move |_|{if form.get_untracked()==original.get_untracked()||web_sys::window().unwrap().confirm_with_message(&t("放弃当前未保存的随笔修改？","Discard the current unsaved note changes?")).unwrap_or(false){reset.run(())}}>{t("取消编辑","Cancel editing")}</button>})}</div></fieldset></form>{move||data.value.get().map(|v|view!{<div class="xcss-table-scroll" tabindex="0" role="region" aria-label=t("数据表格","Data table")><table class="xcss-table" aria-label=t("随笔列表","Note list")><thead><tr>{[t("作者","Author"),t("内容","Content"),t("图片","Image"),t("状态","Status"),t("时间","Time"),t("操作","Actions")].into_iter().map(|l|view!{<th>{l}</th>}).collect_view()}</tr></thead><tbody>{rows(&v,"items").into_iter().map(|item|{let id=num(&item,"id");let edit=item.clone();let toggle=item.clone();view!{<tr><td>{let n=text(&item,"username");if n.is_empty(){t("站长","Site owner")}else{n}}</td><td class="comment-cell">{text(&item,"content")}</td><td>{safe_url(&text(&item,"image_path"),true).map(|u|{let click=u.clone();view!{<button type="button" class="moderation-photo" on:click=move |_|lightbox.set(Some(click.clone()))><img src=u alt=t("查看随笔图片","View note image")/></button>}})}</td><td>{if flag(&item,"is_public"){t("公开","Public")}else{t("私密","Private")}}</td><td>{table_time(&text(&item,"create_time"))}</td><td><div class="xcss-actions"><button class="xcss-button" disabled=move||save.busy.get()||upload.busy.get() on:click=move |_|{if form.get_untracked()!=original.get_untracked()&&!web_sys::window().unwrap().confirm_with_message(&t("放弃当前未保存的随笔修改？","Discard the current unsaved note changes?")).unwrap_or(false){return;}let f=json!({"content":edit["content"],"image_path":edit["image_path"],"is_public":flag(&edit,"is_public")});form.set(f.clone());original.set(f);editing.set(Some(id));filename.set(String::new());}>{t("编辑","Edit")}</button><button class="xcss-button" disabled=move||save.busy.get()||upload.busy.get()||editing.get()==Some(id)&&form.get()!=original.get() on:click=move |_|{let visible=!flag(&toggle,"is_public");let input=json!({"content":toggle["content"],"image_path":toggle["image_path"],"is_public":visible});save.run(format!("/api/v1/content/notes/{id}"),"PUT",Some(input),move|v|{if editing.get_untracked()==Some(id){form.update(|f|f["is_public"]=json!(visible));original.update(|f|f["is_public"]=json!(visible));}
    if num(&v,"id")!=id||flag(&v,"is_public")!=visible{save.error.set(Some(invalid()));return;}data.reload();});}>{if flag(&item,"is_public"){t("隐藏","Hide")}else{t("公开","Publish")}}</button><button class="xcss-button xcss-danger" disabled=move||save.busy.get()||upload.busy.get() on:click=move |_|{if web_sys::window().unwrap().confirm_with_message(&t("确定删除这条随笔？","Delete this note?")).unwrap_or(false){save.run(format!("/api/v1/content/notes/{id}"),"DELETE",None,move |_|{if editing.get_untracked()==Some(id){reset.run(())}data.reload();});}}>{t("删除","Delete")}</button></div></td></tr>}}).collect_view()}</tbody></table></div>})}<ListState data empty=t("暂无随笔","No notes yet")/><Pager data page/><Lightbox src=lightbox/></section>}
}
