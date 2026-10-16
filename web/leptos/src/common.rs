pub fn tr(zh: &str, en: &str, args: &[String]) -> String {
    let mut out = t(zh, en);
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), arg);
    }
    out
}
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::Value;
use wasm_bindgen::JsCast;
pub use xcss_web::api::{ApiError, RequestOptions};
pub use xcss_web::i18n::t;
pub fn text(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}
pub fn num(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}
pub fn flag(v: &Value, k: &str) -> bool {
    v[k].as_bool().unwrap_or_else(|| v[k].as_i64() == Some(1))
}
pub fn rows(v: &Value, k: &str) -> Vec<Value> {
    v.get(k)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}
pub fn list(v: Option<Value>) -> Vec<Value> {
    v.and_then(|v| v.as_array().cloned()).unwrap_or_default()
}
pub fn enc(s: &str) -> String {
    js_sys::encode_uri_component(s)
        .as_string()
        .unwrap_or_default()
}
pub fn navigate(hash: &str) {
    let _ = web_sys::window().unwrap().location().set_hash(hash);
}
pub fn hash() -> String {
    web_sys::window()
        .unwrap()
        .location()
        .hash()
        .unwrap_or_default()
        .trim_start_matches('#')
        .to_string()
}
pub fn pathname() -> String {
    web_sys::window()
        .unwrap()
        .location()
        .pathname()
        .unwrap_or_default()
}
pub fn href(path: &str) -> String {
    let lang = query("lang");
    if lang.is_empty() || path.starts_with('#') {
        return path.into();
    }
    let Some(win) = web_sys::window() else {
        return path.into();
    };
    let Ok(url) = web_sys::Url::new_with_base(path, &win.location().href().unwrap_or_default())
    else {
        return path.into();
    };
    if url.origin() != win.location().origin().unwrap_or_default()
        || !["http:", "https:"].contains(&url.protocol().as_str())
    {
        return path.into();
    }
    url.search_params().set("lang", &xcss_web::i18n::locale());
    format!("{}{}{}", url.pathname(), url.search(), url.hash())
}
pub fn query(key: &str) -> String {
    web_sys::UrlSearchParams::new_with_str(
        &web_sys::window()
            .unwrap()
            .location()
            .search()
            .unwrap_or_default(),
    )
    .ok()
    .and_then(|q| q.get(key))
    .unwrap_or_default()
}
pub fn hash_signal() -> RwSignal<String> {
    let s = RwSignal::new(hash());
    let listener = SendWrapper::new(gloo_events::EventListener::new(
        &web_sys::window().unwrap(),
        "hashchange",
        move |_| s.set(hash()),
    ));
    on_cleanup(move || drop(listener));
    s
}
pub fn search_signal() -> RwSignal<String> {
    let s = RwSignal::new(
        web_sys::window()
            .unwrap()
            .location()
            .search()
            .unwrap_or_default(),
    );
    let listener = SendWrapper::new(gloo_events::EventListener::new(
        &web_sys::window().unwrap(),
        "popstate",
        move |_| {
            s.set(
                web_sys::window()
                    .unwrap()
                    .location()
                    .search()
                    .unwrap_or_default(),
            )
        },
    ));
    on_cleanup(move || drop(listener));
    s
}
pub fn browse(s: RwSignal<String>, changes: &[(&str, Option<String>)]) {
    let url = web_sys::Url::new(
        &web_sys::window()
            .unwrap()
            .location()
            .href()
            .unwrap_or_default(),
    )
    .unwrap();
    let params = url.search_params();
    for (k, v) in changes {
        if let Some(v) = v {
            params.set(k, v)
        } else {
            params.delete(k)
        }
    }
    let relative = format!("{}{}{}", url.pathname(), url.search(), url.hash());
    let _ = web_sys::window()
        .unwrap()
        .history()
        .unwrap()
        .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&relative));
    s.set(url.search());
}
pub fn param(s: RwSignal<String>, key: &str) -> String {
    web_sys::UrlSearchParams::new_with_str(&s.get())
        .ok()
        .and_then(|p| p.get(key))
        .unwrap_or_default()
}
pub fn page_number(s: RwSignal<String>) -> u64 {
    param(s, "page")
        .parse::<u64>()
        .ok()
        .filter(|p| *p > 0)
        .unwrap_or(1)
        .min(100_000)
}
pub fn safe_url(s: &str, blob: bool) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    let url = web_sys::Url::new_with_base(
        s,
        &web_sys::window()
            .unwrap()
            .location()
            .origin()
            .unwrap_or_default(),
    )
    .ok()?;
    (["http:", "https:"].contains(&url.protocol().as_str()) || blob && url.protocol() == "blob:")
        .then(|| url.href())
}
pub fn public_error(e: &ApiError, fallback: &str) -> String {
    if e.code == "display_error" {
        return e.message.clone();
    }
    if [
        "invalid_response",
        "invalid_json_response",
        "invalid_response_shape",
        "invalid_content_type",
    ]
    .contains(&e.code.as_str())
    {
        return t(
            "服务器响应格式无效，无法确认操作结果。",
            "The server response was invalid. The operation result could not be confirmed.",
        );
    }
    match e.status {
        400 => t("请检查输入内容后重试。", "Check the input and try again."),
        401 => t(
            "请检查登录信息，或重新登录后重试。",
            "Check your credentials or sign in again.",
        ),
        403 => t(
            "没有执行此操作的权限。",
            "You do not have permission to perform this action.",
        ),
        404 => t(
            "内容不存在或已被删除。",
            "This content was not found or has been deleted.",
        ),
        409 => t(
            "内容或状态已变化，请刷新后重试。",
            "The content or state has changed. Refresh and try again.",
        ),
        413 => t(
            "上传内容过大，请缩小后重试。",
            "The upload is too large. Reduce its size and try again.",
        ),
        429 => t(
            "请求过于频繁，请稍后重试。",
            "Too many requests. Please try again later.",
        ),
        500.. => t(
            "服务暂时不可用，请稍后重试。",
            "The service is temporarily unavailable. Please try again later.",
        ),
        _ => fallback.into(),
    }
}
pub fn table_time(value: &str) -> String {
    let formatted = crate::dates::format(value);
    if formatted.is_empty() {
        "—".into()
    } else {
        formatted
    }
}
pub fn invalid() -> ApiError {
    ApiError {
        status: 0,
        code: "invalid_response".into(),
        message: t(
            "服务器响应格式无效，无法确认操作结果。",
            "The server response was invalid. The operation result could not be confirmed.",
        ),
        request_id: None,
        retryable: false,
        retry_after_seconds: None,
    }
}
pub async fn request(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
) -> Result<Value, ApiError> {
    let v = xcss_web::api::request_with_options(path, method, body, options).await?;
    if crate::contracts::validate(path, method, &v)
        && (method != "GET" || crate::contracts::matches_page(path, &v))
    {
        Ok(v)
    } else {
        Err(invalid())
    }
}
#[derive(Clone, Copy)]
pub struct Data {
    pub value: RwSignal<Option<Value>>,
    pub previous: RwSignal<Option<Value>>,
    pub error: RwSignal<Option<ApiError>>,
    pub busy: RwSignal<bool>,
    pub revision: RwSignal<u64>,
}
impl Data {
    pub fn reload(self) {
        self.revision.update(|v| *v += 1)
    }
}
pub fn load(path: Signal<String>) -> Data {
    let data = Data {
        value: RwSignal::new(None),
        previous: RwSignal::new(None),
        error: RwSignal::new(None),
        busy: RwSignal::new(false),
        revision: RwSignal::new(0),
    };
    Effect::new(move |_| {
        let path = path.get();
        data.revision.get();
        data.value.set(None);
        data.error.set(None);
        if path.is_empty() {
            data.busy.set(false);
            return;
        }
        data.busy.set(true);
        let controller = web_sys::AbortController::new().unwrap();
        let signal = controller.signal();
        let cleanup = SendWrapper::new(controller);
        on_cleanup(move || cleanup.abort());
        leptos::task::spawn_local(async move {
            let result = request(
                &path,
                "GET",
                None,
                RequestOptions {
                    signal: Some(signal.clone()),
                    max_response_bytes: 4 * 1024 * 1024,
                    ..Default::default()
                },
            )
            .await;
            if signal.aborted() {
                return;
            }
            match result {
                Ok(v) => {
                    data.previous.set(Some(v.clone()));
                    data.value.set(Some(v));
                }
                Err(e) => data.error.set(Some(e)),
            }
            data.busy.set(false);
        });
    });
    data
}
#[derive(Clone, Copy)]
pub struct Action {
    pub busy: RwSignal<bool>,
    pub error: RwSignal<Option<ApiError>>,
    controller: StoredValue<SendWrapper<Option<web_sys::AbortController>>>,
}
impl Action {
    pub fn new() -> Self {
        let s = Self {
            busy: RwSignal::new(false),
            error: RwSignal::new(None),
            controller: StoredValue::new(SendWrapper::new(None)),
        };
        on_cleanup(move || s.abort());
        s
    }
    pub fn abort(self) {
        self.controller.update_value(|c| {
            if let Some(c) = (**c).take() {
                c.abort()
            }
        });
        let _ = self.busy.try_set(false);
    }
    pub fn run(
        self,
        path: String,
        method: &'static str,
        body: Option<Value>,
        done: impl FnOnce(Value) + 'static,
    ) {
        self.run_options(path, method, body, RequestOptions::default(), done)
    }
    pub fn run_options(
        self,
        path: String,
        method: &'static str,
        body: Option<Value>,
        mut opts: RequestOptions,
        done: impl FnOnce(Value) + 'static,
    ) {
        if self.busy.get_untracked() {
            return;
        }
        self.busy.set(true);
        self.error.set(None);
        let c = web_sys::AbortController::new().unwrap();
        let signal = c.signal();
        opts.signal = Some(signal.clone());
        self.controller.set_value(SendWrapper::new(Some(c)));
        leptos::task::spawn_local(async move {
            let r = request(&path, method, body, opts).await;
            if signal.aborted() {
                return;
            }
            match r {
                Ok(v) => done(v),
                Err(mut e) => {
                    if e.status == 409 && path.starts_with("/api/v1/content/resources/") {
                        e.code = "display_error".into();
                        e.message = t(
                            "文件仍被引用或记录已变化。请刷新列表，移除引用后再重试。",
                            "This file is referenced or its record has changed. Refresh the list and remove its references before retrying.",
                        );
                    } else if e.status == 409
                        && method == "PUT"
                        && path.starts_with("/api/v1/content/labels/")
                    {
                        e.code = "display_error".into();
                        e.message = t(
                            "标签仍被文章使用，不能更改分类。请先修改文章的分类和标签。",
                            "This tag is still used by articles. Update their categories and tags before moving this tag.",
                        );
                    }
                    self.error.set(Some(e));
                }
            }
            self.controller.set_value(SendWrapper::new(None));
            self.busy.set(false);
        });
    }
}
#[component]
pub fn Feedback(
    error: RwSignal<Option<ApiError>>,
    #[prop(default=t("请求失败，请重试。","Request failed. Please retry."))] fallback: String,
    #[prop(optional_no_strip)] retry: Option<Callback<()>>,
    #[prop(default = false)] public: bool,
) -> impl IntoView {
    move || {
        error.get().map(|e|{let message=if !public&&retry.is_some(){fallback.clone()}else{public_error(&e,&fallback)};if public{view!{<p role="alert">{message}{retry.map(|retry|view!{<button type="button" on:click=move |_|retry.run(())>{t("重试","Try again")}</button>})}</p>}.into_any()}else{view!{<div class="xcss-error" role="alert">{message}{e.request_id.map(|id|view!{<p class="xcss-request-id">{t("请求标识：","Request ID: ")}<code>{id}</code></p>})}{retry.map(|retry|view!{<button class="xcss-button" type="button" on:click=move |_|retry.run(())>{t("重试","Try again")}</button>})}</div>}.into_any()}})
    }
}
#[component]
pub fn Field(
    label: String,
    key: &'static str,
    form: RwSignal<Value>,
    #[prop(default = false)] textarea: bool,
    #[prop(default = false)] required: bool,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = 32768)] max: u32,
) -> impl IntoView {
    view! {<label class="xcss-form-field"><span class="xcss-field-label">{label}</span>{if textarea{view!{<textarea class="xcss-input" required=required maxlength=max prop:value=move||text(&form.get(),key) on:input=move|ev|form.update(|f|f[key]=Value::String(event_target_value(&ev)))/>}.into_any()}else{view!{<input class="xcss-input" type=kind required=required maxlength=max prop:value=move||{let f=form.get();if kind=="number"{f[key].as_i64().map(|v|v.to_string()).unwrap_or_default()}else{text(&f,key)}} on:input=move|ev|{let v=event_target_value(&ev);form.update(|f|f[key]=if kind=="number"{Value::from(v.parse::<i64>().unwrap_or(0))}else{Value::String(v)})}/>}.into_any()}}</label>}
}
#[component]
pub fn Check(label: String, key: &'static str, form: RwSignal<Value>) -> impl IntoView {
    view! {<label><input type="checkbox" prop:checked=move||flag(&form.get(),key) on:change=move|ev|form.update(|f|f[key]=Value::Bool(event_target_checked(&ev)))/>{label}</label>}
}
#[component]
pub fn Pager(
    data: Data,
    page: RwSignal<u64>,
    #[prop(default=t("分页","Pagination"))] label: String,
    #[prop(default = false)] public: bool,
    #[prop(default = false)] preserve: bool,
    #[prop(default=Signal::derive(||false))] locked: Signal<bool>,
) -> impl IntoView {
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let size = num(&v, "size").max(1);
            let last = ((num(&v, "total") + size - 1) / size).max(1) as u64;
            if page.get_untracked() > last {
                page.set(last);
            }
        }
    });
    move || {
        data.value.get().or_else(||preserve.then(||data.previous.get()).flatten()).filter(|v|num(v,"total")>num(v,"size")||page.get()>1).map(|v|view!{<nav class="pager" aria-label=label.clone()><button type="button" class=if public{""}else{"xcss-button"} disabled=move||locked.get()||(!preserve&&data.busy.get())||page.get()<=1 on:click=move |_|page.update(|p|*p=p.saturating_sub(1).max(1))>{t("上一页","Previous page")}</button><span>{tr("第 {0} 页 · 共 {1} 条","Page {0} · {1} items",&[page.get().to_string(),num(&v,"total").to_string()])}</span><button type="button" class=if public{""}else{"xcss-button"} disabled={move||locked.get()||(!preserve&&data.busy.get())||(page.get() as i64*num(&v,"size"))>=num(&v,"total")} on:click=move |_|page.update(|p|*p+=1)>{t("下一页","Next page")}</button></nav>})
    }
}
#[component]
pub fn Lightbox(src: RwSignal<Option<String>>) -> impl IntoView {
    view! {<For each={move||src.get().into_iter().collect::<Vec<_>>()} key=|s|s.clone() children=move|url|view!{<LightboxDialog url close=Callback::new(move|()|src.set(None))/>}/>}
}
#[component]
fn LightboxDialog(url: String, close: Callback<()>) -> impl IntoView {
    let node = NodeRef::<leptos::html::Dialog>::new();
    Effect::new(move |_| {
        if let Some(node) = node.get() {
            let opener = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .active_element()
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
            let _ = node.show_modal();
            let node = SendWrapper::new(node);
            let opener = SendWrapper::new(opener);
            on_cleanup(move || {
                node.close();
                if let Some(e) = opener.as_ref() {
                    let _ = e.focus();
                }
            });
        }
    });
    view! {<dialog node_ref=node class="image-lightbox" aria-label=t("查看图片","View image") on:cancel=move|ev:web_sys::Event|{ev.prevent_default();close.run(())} on:mousedown=move|ev|{if ev.target()==ev.current_target(){close.run(())}}><button class="image-lightbox-close" type="button" aria-label=t("关闭图片","Close image") on:click=move |_|close.run(())>"×"</button><img src=url alt=t("放大的图片","Enlarged image")/></dialog>}
}
#[component]
pub fn Petals() -> impl IntoView {
    view! {<div class="reference-petals" aria-hidden="true">{(0..18).map(|i|view!{<span style=format!("--petal-left:{}%;--petal-delay:{}s;--petal-duration:{}s;--petal-drift:{}px",(i*37+9)%100,-(i as f64*2.7),16+i%7,if i%2==1{-70}else{90})/>}).collect_view()}</div>}
}
impl Action {
    pub fn upload(self, file: SendWrapper<web_sys::File>, done: impl FnOnce(Value) + 'static) {
        if self.busy.get_untracked() {
            return;
        }
        if file.size() > 10. * 1024. * 1024.
            || !["image/png", "image/jpeg", "image/gif", "image/webp"]
                .contains(&file.type_().as_str())
        {
            self.error.set(Some(ApiError {
                message: t(
                    "请选择不超过 10 MB 的 PNG、JPEG、GIF 或 WebP 图片",
                    "Choose a PNG, JPEG, GIF, or WebP image up to 10 MB",
                ),
                code: "display_error".into(),
                ..invalid()
            }));
            return;
        }
        self.busy.set(true);
        self.error.set(None);
        let c = web_sys::AbortController::new().unwrap();
        let signal = c.signal();
        self.controller.set_value(SendWrapper::new(Some(c)));
        leptos::task::spawn_local(async move {
            let headers = vec![
                ("Content-Type".into(), file.type_()),
                (
                    "X-File-Name".into(),
                    enc(crate::dates::filename_prefix(&file.name())),
                ),
            ];
            let body: web_sys::Blob = (*file).clone().unchecked_into();
            let r = xcss_web::api::request_blob(
                "/api/v1/content/upload",
                "POST",
                body,
                RequestOptions {
                    signal: Some(signal.clone()),
                    headers,
                    ..Default::default()
                },
            )
            .await;
            if signal.aborted() {
                return;
            }
            match r {
                Ok(v) if crate::contracts::upload(&v) => done(v),
                Ok(_) => self.error.set(Some(invalid())),
                Err(e) => self.error.set(Some(e)),
            }
            self.controller.set_value(SendWrapper::new(None));
            self.busy.set(false);
        });
    }
}
#[component]
pub fn FilePicker(
    label: String,
    choose: Callback<SendWrapper<web_sys::File>>,
    #[prop(default=Signal::derive(||false))] disabled: Signal<bool>,
    #[prop(default=Signal::derive(String::new))] filename: Signal<String>,
) -> impl IntoView {
    let input = NodeRef::<leptos::html::Input>::new();
    let id = format!(
        "xocs-file-{}",
        web_sys::window().unwrap().crypto().unwrap().random_uuid()
    );
    view! {<div class="xcss-form-field"><label for=id.clone()>{label.clone()}</label><div class="xocs-file-picker"><input id=id.clone() node_ref=input type="file" hidden disabled=move||disabled.get() accept="image/png,image/jpeg,image/gif,image/webp" on:change=move|ev|{let node=event_target::<web_sys::HtmlInputElement>(&ev);let file=node.files().and_then(|f|f.get(0));node.set_value("");if let Some(file)=file{choose.run(SendWrapper::new(file));}}/><button type="button" class="xcss-button" disabled=move||disabled.get() aria-label=format!("{}{}",t("选择文件：","Choose file: "),label) aria-describedby=format!("{id}-filename") on:click=move |_|{if let Some(input)=input.get(){input.click();}}>{t("选择文件","Choose file")}</button><span class="xocs-file-picker-name" id=format!("{id}-filename") aria-live="polite">{move||{let f=filename.get();if f.is_empty(){t("未选择文件","No file chosen")}else{f}}}</span></div></div>}
}

#[component]
pub fn ListState(data: Data, empty: String, #[prop(default = false)] array: bool) -> impl IntoView {
    move || {
        if data.busy.get() {
            view! {<div class="xcss-loading" role="status">{t("正在加载…","Loading…")}</div>}
                .into_any()
        } else if data.value.get().is_some_and(|v| {
            if array {
                v.as_array().is_some_and(|a| a.is_empty())
            } else {
                rows(&v, "items").is_empty()
            }
        }) {
            view! {<div class="xcss-empty">{empty.clone()}</div>}.into_any()
        } else {
            ().into_any()
        }
    }
}
