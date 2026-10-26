//! HTTP values parsed once before content operations begin.
use axum::http::{HeaderMap, header::ToStrError};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyKeyError {
    #[error("request key header is not ASCII")]
    Header(#[source] ToStrError),
    #[error("request key must contain 1 to 128 visible ASCII bytes")]
    Invalid,
}

pub(super) struct IdempotencyKey<'a>(&'a str);

impl<'a> IdempotencyKey<'a> {
    pub(super) fn from_headers(
        headers: &'a HeaderMap,
    ) -> Result<Option<Self>, IdempotencyKeyError> {
        let Some(value) = headers.get("Idempotency-Key") else {
            return Ok(None);
        };
        let key = value.to_str().map_err(IdempotencyKeyError::Header)?;
        if key.is_empty() || key.len() > 128 || !key.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(IdempotencyKeyError::Invalid);
        }
        Ok(Some(Self(key)))
    }

    pub(super) fn as_str(&self) -> &str {
        self.0
    }
}

#[derive(Deserialize, Serialize)]
#[serde(try_from = "String")]
pub(super) struct ShortContent(String);

#[derive(Debug, thiserror::Error)]
#[error("内容应为 1 至 1024 字")]
pub(super) struct ShortContentError;

impl TryFrom<String> for ShortContent {
    type Error = ShortContentError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let content = raw.trim();
        if content.is_empty() || content.chars().count() > 1024 {
            return Err(ShortContentError);
        }
        Ok(Self(content.to_owned()))
    }
}

impl ShortContent {
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[path = "../../../../test/server/unit/content/input.rs"]
#[cfg(test)]
mod tests;
