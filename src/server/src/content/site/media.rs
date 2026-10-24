//! Optional media selected by the site owner, never bundled fallback assets.
use crate::{AppError, invalid};
use serde::{Deserialize, Serialize};
use url_encoding::address::Url;

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SiteMedia {
    logo: Option<String>,
    journey_background: Option<String>,
    journal_background: Option<String>,
    guestbook_background: Option<String>,
    love_letter_background: Option<String>,
    love_moments_cover: Option<String>,
    love_photos_cover: Option<String>,
    love_wishes_cover: Option<String>,
    toolbox_background: Option<String>,
    toolbox_video: Option<String>,
}

impl SiteMedia {
    pub(super) fn normalize(&mut self) -> Result<(), AppError> {
        for field in [
            &mut self.logo,
            &mut self.journey_background,
            &mut self.journal_background,
            &mut self.guestbook_background,
            &mut self.love_letter_background,
            &mut self.love_moments_cover,
            &mut self.love_photos_cover,
            &mut self.love_wishes_cover,
            &mut self.toolbox_background,
            &mut self.toolbox_video,
        ] {
            normalize_url(field)?;
        }
        Ok(())
    }
}

pub(super) fn normalize_url(field: &mut Option<String>) -> Result<(), AppError> {
    let Some(value) = field.as_deref() else {
        return Ok(());
    };
    if value.chars().any(char::is_control) {
        return Err(invalid("媒体地址无效"));
    }
    let value = value.trim();
    if value.is_empty() {
        *field = None;
        return Ok(());
    }
    if value.len() > 2048 || value.contains('\\') || value.starts_with("//") {
        return Err(invalid("媒体地址无效"));
    }
    let parsed = if value.starts_with('/') {
        Url::parse("https://site.invalid/")
            .expect("static URL")
            .join(value)
    } else if value.split_once("://").is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    }) {
        Url::parse(value)
    } else {
        return Err(invalid("媒体地址必须是 HTTP(S) 或站点绝对路径"));
    };
    if !parsed.is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.has_host()) {
        return Err(invalid("媒体地址无效"));
    }
    *field = Some(value.to_owned());
    Ok(())
}

pub(super) fn normalize_covers(field: &mut Option<String>) -> Result<(), AppError> {
    let Some(value) = field.as_deref() else {
        return Ok(());
    };
    let value = value.trim();
    if value.is_empty() {
        *field = None;
        return Ok(());
    }
    let urls = if value.starts_with('[') {
        serde_json::from_str::<Vec<String>>(value)
            .map_err(|_| invalid("随机封面必须是图片地址数组或每行一个地址"))?
    } else {
        value.lines().map(str::to_owned).collect()
    };
    for url in urls {
        normalize_url(&mut Some(url))?;
    }
    *field = Some(value.to_owned());
    Ok(())
}
