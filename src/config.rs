use anyhow::ensure;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use xcss::config::{EnvMapping, EnvValueKind, Loaded, Override};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub media: Option<PathBuf>,
    pub bind: SocketAddr,
    pub trusted_proxies: Vec<String>,
    pub development: bool,
    pub web: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("/var/lib/isarmg/xocs"),
            log_level: "info".into(),
            media: None,
            bind: "127.0.0.1:8081".parse().expect("static loopback address"),
            trusted_proxies: Vec::new(),
            development: false,
            web: None,
        }
    }
}

#[derive(Args, Clone, Default)]
pub struct Overrides {
    #[arg(skip)]
    pub log_level: Option<String>,
    #[arg(long)]
    pub media: Option<PathBuf>,
    #[arg(long)]
    pub bind: Option<SocketAddr>,
    /// Proxy IPs/CIDRs trusted to supply the shared X-Forwarded-For chain.
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub trusted_proxies: Option<Vec<String>>,
    /// Use insecure HTTP cookies only for an explicit loopback development server.
    #[arg(long, num_args=0..=1, default_missing_value="true")]
    pub development: Option<bool>,
    /// Explicit development-only directory override; production embeds its resources.
    #[arg(long)]
    pub web: Option<PathBuf>,
}

pub fn load(
    config: Option<&Path>,
    data_dir: Option<&Path>,
    args: &Overrides,
) -> anyhow::Result<Loaded<Settings>> {
    let selected_config = default_config_path(config)?;
    let config = selected_config.as_deref();
    let bytes = config.map(xcss::config::read_private_file).transpose()?;
    let mappings = [
        EnvMapping {
            variable: "XOCS_TRUSTED_PROXIES",
            path: "/trusted_proxies",
            kind: EnvValueKind::Json,
        },
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
    let mut cli = Vec::new();
    for (name, path) in [
        ("data_dir", data_dir),
        ("media", args.media.as_deref()),
        ("web", args.web.as_deref()),
    ] {
        if let Some(path) = path {
            cli.push(Override::new(
                format!("/{name}"),
                serde_json::to_value(path)?,
            ));
        }
    }
    if let Some(value) = &args.log_level {
        cli.push(Override::new("/log_level", value.clone()));
    }
    if let Some(bind) = args.bind {
        cli.push(Override::new("/bind", bind.to_string()));
    }
    if let Some(value) = &args.trusted_proxies {
        cli.push(Override::new(
            "/trusted_proxies",
            serde_json::to_value(value)?,
        ));
    }
    if let Some(value) = args.development {
        cli.push(Override::new("/development", value));
    }
    let settings = xcss::config::resolve_validated(
        &Settings::default(),
        bytes.as_deref(),
        &environment,
        &cli,
        |value, source| {
            value.validate().map_err(|_| {
                xcss::config::ConfigError::new(xcss::config::Reason::InvalidValue, "/", source)
            })
        },
    )?;
    settings.value.validate()?;
    if let (Some(config), Ok(media)) = (config, settings.value.media_path())
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
            .unwrap_or_else(|| self.data_dir.join("media")))
    }
    pub fn state_directory(&self) -> anyhow::Result<PathBuf> {
        Ok(self.data_dir.clone())
    }
    fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            matches!(
                self.log_level.as_str(),
                "off" | "error" | "warn" | "info" | "debug" | "trace"
            ),
            "log_level must be off, error, warn, info, debug or trace"
        );
        ensure!(
            self.trusted_proxies.len() <= 32,
            "at most 32 trusted proxy addresses are allowed"
        );
        xcss::client_ip::TrustedProxies::parse(&self.trusted_proxies)?;
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
        if let Some(media) = &self.media {
            ensure!(
                !self.data_dir.starts_with(media)
                    && !media.starts_with(self.data_dir.join("logs"))
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
        }
        ensure!(
            !self.development || self.bind.ip().is_loopback(),
            "development HTTP may only bind loopback"
        );
        ensure!(
            self.web.is_none() || self.development,
            "Web directory overrides require development HTTP"
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
    fn trusted_proxy_configuration_is_opt_in_and_uses_the_shared_parser() {
        assert!(Settings::default().trusted_proxies.is_empty());
        for address in ["127.0.0.1", "::1", "192.0.2.0/24", "2001:db8::/32"] {
            let settings = Settings {
                trusted_proxies: vec![address.into()],
                ..Settings::default()
            };
            assert!(settings.validate().is_ok());
        }
        for address in ["invalid", "192.0.2.0/33", ""] {
            let settings = Settings {
                trusted_proxies: vec![address.into()],
                ..Settings::default()
            };
            assert!(settings.validate().is_err());
        }
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        value["trusted_proxies"] = serde_json::json!([42]);
        assert!(serde_json::from_value::<Settings>(value).is_err());
    }
}

/// An absent implicit file is optional; explicit paths and all other errors fail closed.
fn default_config_path(explicit: Option<&Path>) -> std::io::Result<Option<PathBuf>> {
    if let Some(path) = explicit {
        return Ok(Some(path.to_owned()));
    }
    let path = PathBuf::from("/etc/isarmg/xocs/config.json");
    match std::fs::symlink_metadata(&path) {
        Ok(_) => Ok(Some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod deployment_defaults_tests {
    use super::*;
    #[test]
    fn production_is_default_and_old_configuration_names_are_rejected() {
        assert!(!Settings::default().development);
        assert_eq!(
            Settings::default().data_dir,
            PathBuf::from("/var/lib/isarmg/xocs")
        );
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        json["development_http"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Settings>(json).is_err());
    }
    #[test]
    fn public_and_private_bind_addresses_are_valid_only_in_production() {
        for address in ["10.20.0.12:8081", "0.0.0.0:8081", "[::]:8081"] {
            let mut settings = Settings {
                bind: address.parse().unwrap(),
                ..Settings::default()
            };
            settings.validate().unwrap();
            settings.development = true;
            assert!(settings.validate().is_err());
        }
    }
    #[test]
    fn explicit_missing_config_is_not_silently_dropped() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing.json");
        assert_eq!(
            default_config_path(Some(&path)).unwrap(),
            Some(path.clone())
        );
        assert!(xcss::config::read_private_file(&path).is_err());
    }
}
