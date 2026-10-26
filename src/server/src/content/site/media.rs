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
        let Self {
            logo,
            journey_background,
            journal_background,
            guestbook_background,
            love_letter_background,
            love_moments_cover,
            love_photos_cover,
            love_wishes_cover,
            toolbox_background,
            toolbox_video,
        } = self;
        for field in [
            logo,
            journey_background,
            journal_background,
            guestbook_background,
            love_letter_background,
            love_moments_cover,
            love_photos_cover,
            love_wishes_cover,
            toolbox_background,
            toolbox_video,
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
    }
    .map_err(AppError::MediaUrl)?;
    if !matches!(parsed.scheme(), "http" | "https") || !parsed.has_host() {
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
        serde_json::from_str::<Vec<String>>(value).map_err(AppError::MediaCovers)?
    } else {
        value.lines().map(str::to_owned).collect()
    };
    for url in urls {
        normalize_url(&mut Some(url))?;
    }
    *field = Some(value.to_owned());
    Ok(())
}
