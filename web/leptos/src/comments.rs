use crate::common::*;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::{Value, json};
fn endpoint(kind: &str) -> &'static str {
    match kind {
        "message" => "/api/v1/message-comments",
        "love" => "/api/v1/love-comments",
        _ => "/api/v1/comments",
    }
}
fn commenter(v: &Value) -> String {
    let name = text(v, "username");
    if name.is_empty() {
        t("匿名访客", "Anonymous guest")
    } else {
        name
    }
}
pub fn valid_submission(v: &Value) -> bool {
    let id = text(v, "request_id");
    let b = id.as_bytes();
    b.len() == 36
        && b[14] == b'4'
        && [8, 13, 18, 23].iter().all(|i| b[*i] == b'-')
        && b.iter().enumerate().all(|(i, c)| {
            [8, 13, 18, 23].contains(&i) || c.is_ascii_digit() || (b'a'..=b'f').contains(c)
        })
        && b"89ab".contains(&b[19])
        && v["content"].as_str().is_some_and(|s| !s.trim().is_empty())
        && v.get("parent_comment_id").is_some_and(|v| {
            v.is_null()
                || v.as_u64()
                    .is_some_and(|i| i > 0 && i <= 9_007_199_254_740_991)
        })
        && v.get("article_id").is_none_or(|v| {
            v.as_u64()
                .is_some_and(|i| i > 0 && i <= 9_007_199_254_740_991)
        })
}
fn storage() -> Result<web_sys::Storage, ()> {
    web_sys::window()
        .unwrap()
        .session_storage()
        .map_err(|_| ())?
        .ok_or(())
}
fn read_pending(key: &str) -> Result<Option<Value>, ()> {
    let Some(raw) = storage()?.get_item(key).map_err(|_| ())? else {
        return Ok(None);
    };
    let v = serde_json::from_str(&raw).map_err(|_| ())?;
    if valid_submission(&v) {
        let mut normalized = json!({"request_id":v["request_id"],"content":v["content"],"parent_comment_id":v["parent_comment_id"]});
        if let Some(article) = v.get("article_id") {
            normalized["article_id"] = article.clone();
        }
        Ok(Some(normalized))
    } else {
        Err(())
    }
}
#[component]
pub fn Comments(
    #[prop(default = "article")] kind: &'static str,
    #[prop(default = 0)] article_id: i64,
    #[prop(optional)] title: String,
) -> impl IntoView {
    let path = endpoint(kind);
    let key = format!("xocs.comment-submission:{path}:{article_id}");
    let loaded = read_pending(&key);
    let pending = RwSignal::new(loaded.as_ref().ok().cloned().flatten());
    let storage_failure = RwSignal::new(loaded.is_err());
    let draft = RwSignal::new(
        pending
            .get_untracked()
            .map(|v| text(&v, "content"))
            .unwrap_or_default(),
    );
    let reply = RwSignal::new(None::<Value>);
    let failure = RwSignal::new(String::new());
    let feedback = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let page = RwSignal::new(1u64);
    let url = Signal::derive(move || {
        format!(
            "{path}?page={}&size=10{}",
            page.get(),
            if kind == "article" {
                format!("&article_id={article_id}")
            } else {
                String::new()
            }
        )
    });
    let data = load(url);
    let controller = StoredValue::new(SendWrapper::new(None::<web_sys::AbortController>));
    on_cleanup(move || {
        controller.update_value(|c| {
            if let Some(c) = (**c).take() {
                c.abort()
            }
        })
    });
    let key = StoredValue::new(key);
    let choose = Callback::new(move |v: Value| {
        if !busy.get_untracked()
            && pending.get_untracked().is_none()
            && !storage_failure.get_untracked()
        {
            reply.set(Some(v))
        }
    });
    view! {<section class="article-comments"><h3>{if title.is_empty(){t("评论","Comments")}else{title}}" | "{move||format!("{} {}",data.previous.get().map(|v|num(&v,"total")).unwrap_or(0),t("条评论","comments"))}</h3>{move||(!failure.get().is_empty()).then(||view!{<p role="alert">{failure.get()}</p>})}<Feedback error=data.error fallback=t("评论加载失败","Unable to load comments") public=true retry=Some(Callback::new(move|()|data.reload()))/>{move||data.busy.get().then(||view!{<p role="status">{t("评论加载中…","Loading comments…")}</p>})}{move||storage_failure.get().then(||view!{<p role="alert">{t("无法读取或保存待确认的提交。请先检查评论，再清除待确认状态；当前不会发送新评论。","The pending submission could not be read or saved. Check the comments before clearing it. No new comment will be sent.")}</p>})}{move||(pending.get().is_some()&&failure.get().is_empty()).then(||view!{<p role="status">{t("有一条评论等待确认，请继续确认原提交。","A comment is awaiting confirmation. Continue confirming the original submission.")}</p>})}<For each=move||data.value.get().map(|v|rows(&v,"items")).unwrap_or_default() key=|v|(num(v,"id"),num(v,"reply_count")) children=move|item|view!{<div class="comment-floor"><CommentEntry item=item.clone() reply=choose/><Replies root=item kind choose refresh=data.revision/></div>}/>{move||data.value.get().filter(|v|rows(v,"items").is_empty()).map(|_|view!{<p class="muted">{t("还没有评论。","No comments yet.")}</p>})}<Pager data page public=true preserve=true locked=Signal::derive(move||busy.get()) label=t("评论分页","Comment pages")/><form class="comment-form" on:submit=move|ev|{ev.prevent_default();if busy.get_untracked()||storage_failure.get_untracked(){return;}let retry=pending.get_untracked().is_some();let submission=pending.get_untracked().unwrap_or_else(||{let mut v=json!({"request_id":web_sys::window().unwrap().crypto().unwrap().random_uuid(),"content":draft.get_untracked().trim(),"parent_comment_id":reply.get_untracked().map(|v|v["id"].clone()).unwrap_or(Value::Null)});if kind=="article"{v["article_id"]=json!(article_id);}v});if text(&submission,"content").is_empty(){return;}
    if storage().and_then(|s|s.set_item(&key.get_value(),&submission.to_string()).map_err(|_|())).is_err(){storage_failure.set(true);return;}pending.set(Some(submission.clone()));busy.set(true);failure.set(String::new());feedback.set(String::new());let c=web_sys::AbortController::new().unwrap();let signal=c.signal();controller.set_value(SendWrapper::new(Some(c)));leptos::task::spawn_local(async move{let result=request(path,"POST",Some(submission.clone()),RequestOptions{signal:Some(signal.clone()),..Default::default()}).await;if signal.aborted(){return;}match result{Ok(v)if text(&v,"comment_content")==text(&submission,"content").trim()=>{let _=storage().and_then(|s|s.remove_item(&key.get_value()).map_err(|_|()));pending.set(None);draft.set(String::new());reply.set(None);page.set(1);data.reload();feedback.set(t("评论已发表。","Your comment has been posted."));},Err(e)if !retry&&[400,401,403,404,409,413,429].contains(&e.status)&&e.code!="invalid_error_response"=>{let _=storage().and_then(|s|s.remove_item(&key.get_value()).map_err(|_|()));pending.set(None);failure.set(public_error(&e,&t("请求失败，请重试。","Request failed. Please retry.")));},_=>failure.set(t("评论提交结果尚未确认。请确认原提交；重试会使用同一请求，不会重复发表。","Your comment submission is not yet confirmed. Confirm the original submission; retrying uses the same request and will not post it twice."))}busy.set(false);controller.set_value(SendWrapper::new(None));});}><p class="comment-identity">{t("无需登录，以匿名访客身份发表评论。","Post a comment as an anonymous guest. No account is needed.")}</p>{move||reply.get().map(|v|view!{<button class="text-action" type="button" disabled=move||busy.get()||pending.get().is_some() on:click=move |_|reply.set(None)>{format!("{}{} · {}",t("回复 ","Reply to "),commenter(&v),t("取消","Cancel"))}</button>})}<textarea disabled=move||busy.get()||pending.get().is_some()||storage_failure.get() aria-label=t("写下你的想法","Write your thoughts") required maxlength="1024" prop:value=move||draft.get() on:input=move|ev|draft.set(event_target_value(&ev)) placeholder=t("写下你的想法","Write your thoughts")/><button type="submit" disabled=move||busy.get()||storage_failure.get()||(pending.get().is_none()&&draft.get().trim().is_empty())>{move||if busy.get(){t("发表中…","Posting…")}else if pending.get().is_some(){t("确认原提交","Confirm original submission")}else{t("匿名发表评论","Post anonymous comment")}}</button>{move||(pending.get().is_some()||storage_failure.get()).then(||view!{<button type="button" disabled=move||busy.get() on:click=move |_|{if !web_sys::window().unwrap().confirm_with_message(&t("这条评论可能已经发表。请先检查评论列表；确定放弃确认原提交并清空草稿？","This comment may already be posted. Check the comments first. Stop confirming this submission and clear its draft?")).unwrap_or(false){return;}
    if storage().and_then(|s|s.remove_item(&key.get_value()).map_err(|_|())).is_err(){storage_failure.set(true);return;}pending.set(None);draft.set(String::new());reply.set(None);failure.set(String::new());storage_failure.set(false);}>{t("清除待确认状态","Clear pending submission")}</button>})}{move||(!feedback.get().is_empty()).then(||view!{<p role="status">{feedback.get()}</p>})}</form></section>}
}
#[component]
fn CommentEntry(item: Value, reply: Callback<Value>) -> impl IntoView {
    let target = item.clone();
    view! {<article class="public-comment"><div class="public-comment-head">{safe_url(&text(&item,"avatar"),true).map(|url|view!{<img src=url alt=""/>})}<div><strong>{commenter(&item)}</strong><small>{crate::dates::format(&text(&item,"create_time"))}</small></div></div><p>{(!text(&item,"parent_username").is_empty()).then(||view!{<span class="comment-mention">{format!("@{} ",text(&item,"parent_username"))}</span>})}{text(&item,"comment_content")}</p><div class="comment-actions"><button type="button" on:click=move |_|reply.run(target.clone())>{t("回复","Reply")}</button></div></article>}
}
#[component]
fn Replies(
    root: Value,
    kind: &'static str,
    choose: Callback<Value>,
    refresh: RwSignal<u64>,
) -> impl IntoView {
    let id = num(&root, "id");
    let count = num(&root, "reply_count");
    let page = RwSignal::new(1);
    let path = Signal::derive(move || {
        refresh.get();
        if count == 0 {
            String::new()
        } else {
            format!("{}/{id}/replies?page={}&size=5", endpoint(kind), page.get())
        }
    });
    let data = load(path);
    let items = RwSignal::new(Vec::<Value>::new());
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let next = rows(&v, "items");
            items.update(|current| {
                if page.get_untracked() == 1 {
                    *current = next;
                } else {
                    for n in next {
                        if !current.iter().any(|v| v["id"] == n["id"]) {
                            current.push(n)
                        }
                    }
                }
            });
        }
    });
    view! {<div class="comment-replies"><Feedback error=data.error public=true retry=Some(Callback::new(move|()|data.reload()))/>{move||items.get().into_iter().map(|item|view!{<CommentEntry item reply=choose/>}).collect_view()}{move||data.value.get().filter(|v|(items.get().len() as i64)<num(v,"total")).map(|v|view!{<button type="button" class="comment-more" disabled=move||data.busy.get() on:click=move |_|page.update(|p|*p+=1)>{format!("{} {} {}",t("展开剩余","Show"),num(&v,"total")-items.get().len() as i64,t("条回复","more replies"))}</button>})}</div>}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_identity_and_unicode_trim() {
        assert!(valid_submission(
            &json!({"request_id":"12345678-1234-4123-8123-123456789abc","content":"hello","parent_comment_id":null})
        ));
        assert!(!valid_submission(
            &json!({"request_id":"12345678-1234-7123-8123-123456789abc","content":"hello","parent_comment_id":null})
        ));
        assert_eq!("\u{85} text \u{85}".trim(), "text");
    }
}
