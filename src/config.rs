use anyhow::ensure;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use xcss::config::{EnvMapping, EnvValueKind, Loaded};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(skip)]
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub log_level: String,
    pub media: Option<PathBuf>,
    pub bind: SocketAddr,
    pub development: bool,
    pub web: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/opt/isarmg/xocs"),
            data_dir: PathBuf::from("/opt/isarmg/xocs/state"),
            log_level: "info".into(),
            media: None,
            bind: "127.0.0.1:8081".parse().expect("static loopback address"),
            development: false,
            web: None,
        }
    }
}

pub fn load(root: &Path, config: &Path) -> anyhow::Result<Loaded<Settings>> {
    ensure!(
        root.is_absolute()
            && root != Path::new("/")
            && !root.components().any(|part| matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )),
        "ROOT must be an absolute normalized directory"
    );
    let bytes = xcss::config::read_private_file(config)?;
    let mappings = [
        EnvMapping {
            variable: "XOCS_DATA_DIR",
            path: "/data_dir",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "XOCS_LOG_LEVEL",
            path: "/log_level",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "XOCS_MEDIA",
            path: "/media",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "XOCS_BIND",
            path: "/bind",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "XOCS_DEVELOPMENT",
            path: "/development",
            kind: EnvValueKind::Boolean,
        },
        EnvMapping {
            variable: "XOCS_DEV_WEB_DIR",
            path: "/web",
            kind: EnvValueKind::String,
        },
    ];
    let mut values = std::collections::BTreeMap::new();
    for mapping in &mappings {
        match std::env::var(mapping.variable) {
            Ok(value) => {
                values.insert(mapping.variable, value);
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(_) => anyhow::bail!("{} must be valid Unicode", mapping.variable),
        }
    }
    let environment = xcss::config::read_environment(&mappings, |name| values.get(name).cloned())?;
    let defaults = Settings {
        root: root.to_path_buf(),
        data_dir: root.join("state"),
        ..Default::default()
    };
    let mut settings = xcss::config::resolve_validated(
        &defaults,
        Some(&bytes),
        &environment,
        &[],
        |value, source| {
            let mut effective = value.clone();
            effective.root = root.to_path_buf();
            effective.validate().map_err(|_| {
                xcss::config::ConfigError::new(xcss::config::Reason::InvalidValue, "/", source)
            })
        },
    )?;
    settings.value.root = root.to_path_buf();
    settings.value.validate()?;
    if let Ok(media) = settings.value.media_path()
        && config.starts_with(media)
    {
        return Err(xcss::config::ConfigError::new(
            xcss::config::Reason::InvalidValue,
            "/media",
            xcss::config::ConfigSource::File,
        )
        .into());
    }
    Ok(settings)
}

impl Settings {
    pub fn database_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self.data_dir.join("site.sqlite"))
    }
    pub fn media_path(&self) -> anyhow::Result<PathBuf> {
        Ok(self
            .media
            .clone()
            .unwrap_or_else(|| self.root.join("media")))
    }
    pub fn state_directory(&self) -> anyhow::Result<PathBuf> {
        Ok(self.data_dir.clone())
    }
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            matches!(
                self.log_level.as_str(),
                "off" | "error" | "warn" | "info" | "debug" | "trace"
            ),
            "log_level must be off, error, warn, info, debug or trace"
        );
        for path in std::iter::once(&self.data_dir)
            .chain(self.media.iter())
            .chain(self.web.iter())
        {
            ensure!(path.is_absolute(), "configured paths must be absolute");
            ensure!(
                !path.components().any(|p| matches!(
                    p,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )),
                "configured paths must not contain traversal components"
            );
        }
        for selected in [&self.data_dir, &self.media_path()?] {
            for name in [
                "bin",
                "libexec",
                "share",
                "config",
                "logs",
                "run",
                "maintenance",
            ] {
                let protected = self.root.join(name);
                ensure!(
                    !selected.starts_with(&protected) && !protected.starts_with(selected),
                    "state and media cannot overlap private or immutable ROOT namespaces"
                );
            }
        }
        let media = self.media_path()?;
        ensure!(
            !self.data_dir.starts_with(&media)
                && !media
                    .strip_prefix(&self.data_dir)
                    .ok()
                    .and_then(|path| path.components().next())
                    .and_then(|part| part.as_os_str().to_str())
                    .is_some_and(|name| name.starts_with("site.sqlite")
                        || name.starts_with(".state-")
                        || name.starts_with(".xocs-init-")),
            "media directory must not expose private application state"
        );
        ensure!(
            !self.development || self.bind.ip().is_loopback(),
            "development HTTP may only bind loopback"
        );
        ensure!(
            self.web.is_none()
                || (self.development
                    && cfg!(debug_assertions)
                    && env!("XOCS_SOURCE_REVISION") == "unbound"),
            "Web directory overrides require an unbound development build"
        );
        Ok(())
    }
}

pub fn runtime_lock(settings: &Settings) -> anyhow::Result<xcss::state_file::InstanceLock> {
    let state = xcss::state_file::PrivateStateDirectory::open(settings.state_directory()?)?;
    xcss::server_cli::runtime_allowed(state.path()).map_err(crate::CliFailure)?;
    let lock = state.try_instance_lock()?;
    xcss::server_cli::runtime_allowed(state.path()).map_err(crate::CliFailure)?;
    state.verify_identity()?;
    Ok(lock)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_has_one_root_and_rejects_retired_proxy_configuration() {
        assert_eq!(
            Settings::default().data_dir,
            PathBuf::from("/opt/isarmg/xocs/state")
        );
        for field in ["trusted_proxies", "development_http"] {
            let mut document = serde_json::to_value(Settings::default()).unwrap();
            document[field] = serde_json::json!(true);
            assert!(serde_json::from_value::<Settings>(document).is_err());
        }
    }
    #[test]
    fn selected_root_controls_default_media_and_private_namespace_validation() {
        let root = PathBuf::from("/srv/test-xocs");
        let settings = Settings {
            root: root.clone(),
            data_dir: root.join("state"),
            ..Default::default()
        };
        assert_eq!(settings.media_path().unwrap(), root.join("media"));
        settings.validate().unwrap();
        for namespace in [
            "bin",
            "libexec",
            "share",
            "config",
            "logs",
            "run",
            "maintenance",
        ] {
            let invalid = Settings {
                media: Some(root.join(namespace).join("public")),
                ..settings.clone()
            };
            assert!(invalid.validate().is_err());
            let invalid = Settings {
                data_dir: root.join(namespace).join("state"),
                ..settings.clone()
            };
            assert!(invalid.validate().is_err());
        }
        let external = Settings {
            media: Some(PathBuf::from("/srv/test-xocs-media")),
            data_dir: PathBuf::from("/srv/test-xocs-data"),
            ..settings
        };
        external.validate().unwrap();
    }

    #[test]
    fn development_requires_loopback() {
        for address in ["10.20.0.12:8081", "0.0.0.0:8081", "[::]:8081"] {
            let mut value = Settings {
                bind: address.parse().unwrap(),
                ..Default::default()
            };
            value.validate().unwrap();
            value.development = true;
            assert!(value.validate().is_err());
        }
    }
}
