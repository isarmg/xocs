//! One URL policy for article attachments, video fences and editor insertion.
//! Never accept author HTML, player attributes, arbitrary iframe origins, or
//! provider parameters capable of changing navigation or autoplay behavior.

const QUERY_KEYS: [&str; 6] = ["v", "bvid", "avid", "aid", "p", "page"];

struct Address {
    href: String,
    scheme: String,
    host: String,
    path: String,
    port: String,
    credentials: bool,
    query: Vec<(String, String)>,
}

impl Address {
    fn query(&self, key: &str) -> Option<&str> {
        let mut values = self.query.iter().filter(|(name, _)| name == key);
        let (_, value) = values.next()?;
        values.next().is_none().then_some(value.as_str())
    }
}

#[cfg(target_arch = "wasm32")]
fn address(source: &str) -> Option<Address> {
    let url = web_sys::Url::new_with_base(source, "https://article.invalid/").ok()?;
    let parameters = url.search_params();
    let query = QUERY_KEYS
        .iter()
        .flat_map(|key| {
            let values = parameters.get_all(key);
            (0..values.length()).map(move |index| {
                let value = values
                    .get(index)
                    .as_string()
                    .expect("invariant: URLSearchParams contains strings");
                ((*key).into(), value)
            })
        })
        .collect();
    Some(Address {
        href: url.href(),
        scheme: url.protocol().trim_end_matches(':').into(),
        host: url.hostname(),
        path: url.pathname(),
        port: url.port(),
        credentials: !url.username().is_empty() || !url.password().is_empty(),
        query,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn address(source: &str) -> Option<Address> {
    let base = url_encoding::address::Url::parse("https://article.invalid/")
        .expect("invariant: article base URL is valid");
    let url = base.join(source).ok()?;
    Some(Address {
        href: url.as_str().into(),
        scheme: url.scheme().into(),
        host: url.host_str()?.into(),
        path: url.path().into(),
        port: url.port().map(|port| port.to_string()).unwrap_or_default(),
        credentials: !url.username().is_empty() || url.password().is_some(),
        query: url
            .query_pairs()
            .filter(|(key, _)| QUERY_KEYS.contains(&key.as_ref()))
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect(),
    })
}

fn parse(source: &str) -> Option<Address> {
    if source.is_empty()
        || source.chars().any(|c| c.is_whitespace() || c.is_control())
        || source.contains(['\\', '`'])
        || !(source.starts_with('/') && !source.starts_with("//")
            || source.split_once("://").is_some_and(|(scheme, _)| {
                scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
            }))
    {
        return None;
    }
    let mut url = address(source)?;
    if !["http", "https"].contains(&url.scheme.as_str()) || url.host.is_empty() || url.credentials {
        return None;
    }
    if source.starts_with('/') {
        url.href = url.href.strip_prefix("https://article.invalid")?.into();
        // Dot-segment normalization can expose a second leading slash. Such
        // a serialized root path would become an authority on the next parse.
        if url.href.starts_with("//") {
            return None;
        }
    }
    Some(url)
}

fn escaped(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn youtube_id(value: &str) -> bool {
    value.len() == 11
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}

fn bvid(value: &str) -> bool {
    value.len() == 12 && value.starts_with("BV") && value.bytes().all(|c| c.is_ascii_alphanumeric())
}

fn positive_integer(value: &str) -> Option<u64> {
    (!value.is_empty() && value.bytes().all(|c| c.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
        .filter(|number| *number > 0)
}

fn iframe(source: &str, title: &str) -> String {
    format!(
        "<iframe class=\"article-video-embed article-media\" src=\"{}\" title=\"{}\" loading=\"lazy\" referrerpolicy=\"strict-origin-when-cross-origin\" sandbox=\"allow-scripts allow-same-origin allow-presentation\" allow=\"fullscreen; picture-in-picture; encrypted-media\" allowfullscreen></iframe>\n",
        escaped(source),
        escaped(title),
    )
}

fn platform_player(url: &Address) -> Option<String> {
    // Standard ports disappear during URL normalization. A nonstandard port
    // must never be treated as one of the admitted provider origins.
    if !url.port.is_empty()
        || QUERY_KEYS
            .iter()
            .any(|key| url.query.iter().filter(|(name, _)| name == key).count() > 1)
    {
        return None;
    }
    match url.host.as_str() {
        "youtube.com"
        | "www.youtube.com"
        | "m.youtube.com"
        | "youtube-nocookie.com"
        | "www.youtube-nocookie.com"
        | "youtu.be" => {
            let id = if url.host == "youtu.be" {
                url.path
                    .strip_prefix('/')?
                    .strip_suffix('/')
                    .unwrap_or(url.path.strip_prefix('/')?)
            } else if url.path == "/watch" {
                url.query("v")?
            } else {
                url.path
                    .strip_prefix("/embed/")
                    .or_else(|| url.path.strip_prefix("/shorts/"))
                    .or_else(|| url.path.strip_prefix("/live/"))?
            };
            if !youtube_id(id) {
                return None;
            }
            Some(iframe(
                &format!("https://www.youtube-nocookie.com/embed/{id}"),
                "YouTube video player",
            ))
        }
        "bilibili.com" | "www.bilibili.com" | "m.bilibili.com" | "player.bilibili.com" => {
            if url.query("p").is_some() && url.query("page").is_some() {
                return None;
            }
            let mut source = if url.host == "player.bilibili.com" && url.path == "/player.html" {
                if let Some(id) = url.query("bvid").filter(|id| bvid(id)) {
                    format!("https://player.bilibili.com/player.html?bvid={id}")
                } else {
                    let id = positive_integer(url.query("aid").or_else(|| url.query("avid"))?)?;
                    format!("https://player.bilibili.com/player.html?aid={id}")
                }
            } else {
                let id = url.path.strip_prefix("/video/")?.trim_end_matches('/');
                if bvid(id) {
                    format!("https://player.bilibili.com/player.html?bvid={id}")
                } else {
                    let id = positive_integer(id.strip_prefix("av")?)?;
                    format!("https://player.bilibili.com/player.html?aid={id}")
                }
            };
            if let Some(page) = url.query("p").or_else(|| url.query("page")) {
                source.push_str(&format!("&page={}", positive_integer(page)?));
            }
            source.push_str("&autoplay=0");
            Some(iframe(&source, "Bilibili video player"))
        }
        _ => None,
    }
}

/// Render a single validated native or provider player. Markdown calls this
/// only for an exact `video` fence; attachments use the same URL policy.
pub(crate) fn video_html(source: &str, poster: Option<&str>) -> Option<String> {
    let source = source.trim();
    let url = parse(source)?;
    let extension = url
        .path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase());
    if extension
        .as_deref()
        .is_some_and(|extension| ["mp4", "webm"].contains(&extension))
    {
        let poster = poster
            .and_then(|poster| parse(poster.trim()))
            .map(|poster| format!(" poster=\"{}\"", escaped(&poster.href)))
            .unwrap_or_default();
        return Some(format!(
            "<video class=\"article-video article-media\" controls playsinline preload=\"metadata\" aria-label=\"Video player\"{poster} src=\"{}\"></video>\n",
            escaped(&url.href),
        ));
    }
    platform_player(&url)
}

/// A fixed fence keeps URL insertion independent of authored HTML. Rejecting
/// line breaks/backticks prevents a supplied URL from terminating the fence.
pub(crate) fn video_block(source: &str) -> Option<String> {
    let source = source.trim();
    video_html(source, None)?;
    Some(format!("```video\n{source}\n```"))
}
