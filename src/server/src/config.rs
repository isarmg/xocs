use crate::Error;
use blog_application::config::{EnvMapping, EnvValueKind, Loaded};
use std::path::{Path, PathBuf};

pub use crate::config_definition::Settings;

pub fn load(root: &Path, config: &Path) -> Result<Loaded<Settings>, Error> {
    load_with_environment(root, config, true)
}

/// Offline operations take their complete state target from ROOT/config only.
/// Runtime environment overrides must never redirect a maintenance operation.
#[cfg(feature = "maintenance")]
pub(crate) fn load_offline(root: &Path, config: &Path) -> Result<Loaded<Settings>, Error> {
    load_with_environment(root, config, false)
}

fn load_with_environment(
    root: &Path,
    config: &Path,
    runtime: bool,
) -> Result<Loaded<Settings>, Error> {
    if !root.is_absolute()
        || root == Path::new("/")
        || root.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(Error::InvalidRoot);
    }
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
                Err(source) => {
                    return Err(Error::Environment {
                        variable: mapping.variable,
                        source,
                    });
                }
            }
        }
    }
    let environment =
        blog_application::config::read_environment(&mappings, |name| values.get(name).cloned())?;
    let defaults = Settings::for_root(root);
    let mut settings = blog_application::config::resolve_validated(
        &defaults,
        Some(&bytes),
        &environment,
        &[],
        |value, source| value.validate_at(root, source),
    )?;
    settings.value.root = root.to_path_buf();
    if config.starts_with(settings.value.media_path()) {
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
    pub fn database_path(&self) -> PathBuf {
        self.data_dir
            .join(crate::config_definition::DATABASE_FILENAME)
    }
    pub fn media_path(&self) -> PathBuf {
        self.media
            .clone()
            .unwrap_or_else(|| self.root.join(crate::config_definition::MEDIA_DIRECTORY))
    }
    #[cfg(test)]
    pub fn validate(&self) -> Result<(), blog_application::config::ConfigError> {
        self.validate_at(&self.root, blog_application::config::ConfigSource::File)
    }

    fn validate_at(
        &self,
        root: &Path,
        source: blog_application::config::ConfigSource,
    ) -> Result<(), blog_application::config::ConfigError> {
        let source_bound = !cfg!(debug_assertions) || env!("BLOG_SOURCE_REVISION") != "unbound";
        blog_application::configuration_contract::validate_rules(
            &crate::config_definition::current_rules(source_bound),
            &serde_json::to_value(self).expect("invariant: settings serialize to JSON"),
            root,
            source,
        )
    }
}

pub fn runtime_lock(
    settings: &Settings,
) -> Result<blog_application::state_file::InstanceLock, Error> {
    let state = blog_application::state_file::PrivateStateDirectory::open(&settings.data_dir)?;
    let lock = state.try_instance_lock()?;
    state.verify_identity()?;
    Ok(lock)
}

#[path = "../../../test/server/unit/config/tests.rs"]
#[cfg(test)]
mod tests;
