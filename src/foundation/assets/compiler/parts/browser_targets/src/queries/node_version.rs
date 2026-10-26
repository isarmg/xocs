use std::cmp::Ordering;

use crate::browser_targets::{error::Error, semver::Version};

pub(super) struct NodeVersion {
    major: u16,
    minor: Option<u16>,
    patch: Option<u16>,
}

impl NodeVersion {
    pub(super) fn parse(raw: &str) -> Result<Self, Error> {
        let mut components = raw.split('.').map(|component| {
            component
                .parse::<u16>()
                .map_err(|source| Error::InvalidNodejsVersion {
                    version: raw.to_owned(),
                    source,
                })
        });
        let major = components
            .next()
            .expect("invariant: str::split always yields its first segment")?;
        let minor = components.next().transpose()?;
        let patch = components.next().transpose()?;
        if components.next().transpose()?.is_some() {
            return Err(Error::UnknownNodejsVersion(raw.to_owned()));
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub(super) fn matches(&self, version: Version) -> bool {
        self.major == version.0
            && self.minor.is_none_or(|minor| minor == version.1)
            && self.patch.is_none_or(|patch| patch == version.2)
    }

    pub(super) fn padded(&self) -> Version {
        Version(self.major, self.minor.unwrap_or(0), self.patch.unwrap_or(0))
    }

    // Browserslist's bounded Node ranges compare major and optional minor only.
    pub(super) fn compare_to(&self, version: Version) -> Ordering {
        self.major.cmp(&version.0).then_with(|| {
            self.minor
                .map_or(Ordering::Equal, |minor| minor.cmp(&version.1))
        })
    }
}
