use anyhow::ensure;
use blog_application::config::{EnvMapping, EnvValueKind, Loaded};
use std::path::{Path, PathBuf};

pub use crate::config_definition::Settings;

pub fn load(root: &Path, config: &Path) -> anyhow::Result<Loaded<Settings>> {
    load_with_environment(root, config, true)
}

/// Offline operations take their complete state target from ROOT/config only.
/// Runtime environment overrides must never redirect a maintenance operation.
#[cfg(feature = "maintenance")]
pub(crate) fn load_offline(root: &Path, config: &Path) -> anyhow::Result<Loaded<Settings>> {
    load_with_environment(root, config, false)
}

fn load_with_environment(
    root: &Path,
    config: &Path,
    runtime: bool,
) -> anyhow::Result<Loaded<Settings>> {
    ensure!(
        root.is_absolute()
            && root != Path::new("/")
            && !root.components().any(|part| matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )),
        "ROOT must be an absolute normalized directory"
    );
    let bytes = blog_application::config::read_managed_file(
        config,
        cfg!(debug_assertions) && env!("BLOG_SOURCE_REVISION") == "unbound",
    )?;
    let mappings = [
        EnvMapping {
            variable: "BLOG_DATA_DIR",
            path: "/data_dir",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "BLOG_LOG_LEVEL",
            path: "/log_level",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "BLOG_MEDIA",
            path: "/media",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "BLOG_BIND",
            path: "/bind",
            kind: EnvValueKind::String,
        },
        EnvMapping {
            variable: "BLOG_DEVELOPMENT",
            path: "/development",
            kind: EnvValueKind::Boolean,
        },
        EnvMapping {
            variable: "BLOG_DEV_WEB_DIR",
            path: "/web",
            kind: EnvValueKind::String,
        },
    ];
    let mut values = std::collections::BTreeMap::new();
    if runtime {
        for mapping in &mappings {
            match std::env::var(mapping.variable) {
                Ok(value) => {
                    values.insert(mapping.variable, value);
                }
                Err(std::env::VarError::NotPresent) => {}
                Err(_) => anyhow::bail!("{} must be valid Unicode", mapping.variable),
            }
        }
    }
    let environment =
        blog_application::config::read_environment(&mappings, |name| values.get(name).cloned())?;
    let defaults = Settings {
        root: root.to_path_buf(),
        data_dir: root.join("state"),
        ..Default::default()
    };
    let mut settings = blog_application::config::resolve_validated(
        &defaults,
        Some(&bytes),
        &environment,
        &[],
        |value, source| {
            let mut effective = value.clone();
            effective.root = root.to_path_buf();
            effective.validate().map_err(|_| {
                blog_application::config::ConfigError::new(
                    blog_application::config::Reason::InvalidValue,
                    "/",
                    source,
                )
            })
        },
    )?;
    settings.value.root = root.to_path_buf();
    settings.value.validate()?;
    if let Ok(media) = settings.value.media_path()
        && config.starts_with(media)
    {
        return Err(blog_application::config::ConfigError::new(
            blog_application::config::Reason::InvalidValue,
            "/media",
            blog_application::config::ConfigSource::File,
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
        let source_bound = !cfg!(debug_assertions) || env!("BLOG_SOURCE_REVISION") != "unbound";
        blog_application::configuration_contract::validate_rules(
            &crate::config_definition::current_rules(source_bound),
            &serde_json::to_value(self)?,
            &self.root,
            blog_application::config::ConfigSource::File,
        )?;
        Ok(())
    }
}

pub fn runtime_lock(
    settings: &Settings,
) -> anyhow::Result<blog_application::state_file::InstanceLock> {
    let state =
        blog_application::state_file::PrivateStateDirectory::open(settings.state_directory()?)?;
    let lock = state.try_instance_lock()?;
    state.verify_identity()?;
    Ok(lock)
}

#[path = "../../../test/server/unit/config/tests.rs"]
#[cfg(test)]
mod tests;
