//! Link static ESM, CSS and literal import.meta.url assets into a bounded web bundle.
use crate::inventory::{self, Files};
use crate::layout;
use crate::source_inputs;
use compression::streams::{Compression, GzBuilder};
use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
mod css_error;
mod error;

use css_error::owned_css_error;
pub use error::{AssetError, BudgetError, ConfigurationError, ImportError};

type Result<T, E = AssetError> = std::result::Result<T, E>;

fn input_error(path: &Path, source: io::Error) -> AssetError {
    AssetError::Input {
        path: path.into(),
        source,
    }
}

fn utf8(bytes: Vec<u8>, path: &Path) -> Result<String> {
    String::from_utf8(bytes).map_err(|source| AssetError::Utf8 {
        path: path.into(),
        source,
    })
}

fn canonicalize(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|source| AssetError::Canonicalize {
            path: path.into(),
            source,
        })
}

fn source_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(AssetError::Inspect {
            path: path.into(),
            source,
        }),
    }
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("invariant: a string serializes as JSON")
}

fn source_stem(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .expect("invariant: a validated source file has a UTF-8 stem")
}

const MAX_PUBLIC_MEDIA_BYTES: usize = 4 * 1024 * 1024;
const MAX_PUBLIC_MEDIA_TOTAL_BYTES: usize = 8 * 1024 * 1024;

#[derive(Default)]
struct OutputBudget {
    bundle: usize,
    media: usize,
}

impl OutputBudget {
    fn add(&mut self, config: &Web, name: &str, size: usize, public_media: bool) -> Result<()> {
        if name.ends_with(".map") {
            return Err(AssetError::SourceMap);
        }
        if public_media {
            if size > MAX_PUBLIC_MEDIA_BYTES {
                return Err(BudgetError::Media {
                    name: name.into(),
                    limit: MAX_PUBLIC_MEDIA_BYTES,
                    size,
                }
                .into());
            }
            let total = self
                .media
                .checked_add(size)
                .ok_or(BudgetError::CountOverflow)?;
            if total > MAX_PUBLIC_MEDIA_TOTAL_BYTES {
                return Err(BudgetError::MediaTotal {
                    limit: MAX_PUBLIC_MEDIA_TOTAL_BYTES,
                    total,
                }
                .into());
            }
            self.media = total;
        } else {
            let limit = if name.ends_with(".woff") || name.ends_with(".woff2") {
                config.max_font_bytes
            } else if name.ends_with(".wasm") || name.ends_with(".wasm.gz") {
                config.max_wasm_bytes
            } else {
                config.max_asset_bytes
            };
            if size > limit {
                return Err(BudgetError::Asset {
                    name: name.into(),
                    limit,
                    size,
                }
                .into());
            }
            let total = self
                .bundle
                .checked_add(size)
                .ok_or(BudgetError::CountOverflow)?;
            if total > config.max_total_bytes {
                return Err(BudgetError::Bundle {
                    limit: config.max_total_bytes,
                    total,
                }
                .into());
            }
            self.bundle = total;
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicAssetsManifest {
    format: u32,
    #[serde(deserialize_with = "unique_public_files")]
    files: Files,
}

fn unique_public_files<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Files, D::Error> {
    struct UniqueFiles;
    impl<'de> serde::de::Visitor<'de> for UniqueFiles {
        type Value = Files;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a unique public-relative-path to SHA-256 map")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(self, mut map: M) -> Result<Files, M::Error> {
            let mut files = Files::new();
            while let Some((name, digest)) = map.next_entry::<String, String>()? {
                if files.insert(name.clone(), digest).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "Duplicate public asset: {name}"
                    )));
                }
            }
            Ok(files)
        }
    }
    deserializer.deserialize_map(UniqueFiles)
}

fn public_manifest_path(web: &Path, name: &str) -> Result<PathBuf> {
    let name = Path::new(name);
    inventory::relative(name)?;
    if name.components().count() != 1 || !name.extension().is_some_and(|ext| ext == "json") {
        return Err(ConfigurationError::ManifestName.into());
    }
    Ok(web.join(name))
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub web: Web,
    pub aliases: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            web: Web::default(),
            aliases: BTreeMap::from([("@blog/styles/".into(), "styles/".into())]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputKind {
    Spa,
    Embedded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeRoot {
    Web,
    Product,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Web {
    pub kind: OutputKind,
    pub entry: String,
    pub entries: BTreeMap<String, String>,
    pub copies: BTreeMap<String, String>,
    pub css_output: String,
    pub relative_urls: bool,
    pub runtime_root: RuntimeRoot,
    pub dist: String,
    pub base: String,
    pub max_asset_bytes: usize,
    pub max_wasm_bytes: usize,
    pub max_font_bytes: usize,
    pub max_total_bytes: usize,
    pub public_assets_manifest: String,
    pub licenses: Vec<String>,
    pub proxies: BTreeMap<String, String>,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            kind: OutputKind::Spa,
            entry: String::new(),
            entries: BTreeMap::new(),
            copies: BTreeMap::new(),
            css_output: String::new(),
            relative_urls: false,
            runtime_root: RuntimeRoot::Web,
            dist: format!("{}/dist", layout::WEB),
            base: "/".into(),
            max_asset_bytes: 512 * 1024,
            max_wasm_bytes: 4 * 1024 * 1024,
            max_font_bytes: 16 * 1024 * 1024,
            max_total_bytes: 16 * 1024 * 1024,
            public_assets_manifest: String::new(),
            licenses: vec![
                "styles/web-fonts/OFL.txt".into(),
                "styles/web-fonts/CJK-LICENSE.txt".into(),
                "styles/web-fonts/NORMAL-LICENSE.txt".into(),
            ],
            proxies: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn read(web: &Path) -> Result<Self> {
        let path = web.join("rust-web.toml");
        let mut config: Self = match inventory::read(&path) {
            Ok(bytes) => toml::from_str(&utf8(bytes, &path)?).map_err(|source| {
                AssetError::Configuration {
                    path: path.clone(),
                    source: Box::new(source),
                }
            })?,
            Err(inventory::InventoryError::Read { source, .. })
                if source.kind() == io::ErrorKind::NotFound =>
            {
                Self::default()
            }
            Err(error) => return Err(error.into()),
        };
        if config.web.entry.is_empty() {
            config.web.entry = "src/main.js".into();
        }
        source_inputs::web_relative(Path::new(&config.web.entry), false)
            .map_err(|source| input_error(Path::new(&config.web.entry), source))?;
        if config.web.kind == OutputKind::Embedded && config.web.entries.is_empty() {
            return Err(ConfigurationError::EmbeddedEntries.into());
        }
        for (output, source) in config.web.entries.iter().chain(config.web.copies.iter()) {
            inventory::relative(Path::new(output))?;
            source_inputs::web_relative(Path::new(source), false)
                .map_err(|error| input_error(Path::new(source), error))?;
        }
        for license in &config.web.licenses {
            source_inputs::web_relative(Path::new(license), false)
                .map_err(|source| input_error(Path::new(license), source))?;
        }
        if !config.web.css_output.is_empty() {
            inventory::relative(Path::new(&config.web.css_output))?;
        }
        if !config.web.public_assets_manifest.is_empty() {
            public_manifest_path(web, &config.web.public_assets_manifest)?;
        }
        if !(config.web.base.starts_with('/')
            && config.web.base.ends_with('/')
            && !config
                .web
                .base
                .contains(['\\', '?', '#', '"', '\'', '<', '>', '\n', '\r'])
            && !config.web.base.contains("..")
            && !config.web.base.starts_with("//"))
        {
            return Err(ConfigurationError::BaseUrl.into());
        }
        for limit in [
            config.web.max_asset_bytes,
            config.web.max_wasm_bytes,
            config.web.max_font_bytes,
            config.web.max_total_bytes,
        ] {
            if !(1..=64 * 1024 * 1024).contains(&limit) {
                return Err(ConfigurationError::BudgetRange.into());
            }
        }
        // Raising an established project budget requires a separately reviewed code change.
        if !(config.web.max_asset_bytes <= 512 * 1024
            && config.web.max_wasm_bytes <= 4 * 1024 * 1024
            && config.web.max_font_bytes <= 16 * 1024 * 1024
            && config.web.max_total_bytes <= 16 * 1024 * 1024)
        {
            return Err(ConfigurationError::RelaxedBudget.into());
        }
        for destination in config.aliases.values() {
            let destination = Path::new(destination.trim_end_matches('/'));
            source_inputs::web_relative(destination, true)
                .map_err(|source| input_error(destination, source))?;
        }
        Ok(config)
    }
}

#[derive(Clone)]
struct Module {
    source: PathBuf,
    output: String,
    imports: BTreeSet<PathBuf>,
    derived_outputs: Vec<String>,
}

pub struct Linker {
    web: PathBuf,
    pkg: PathBuf,
    pub config: Config,
    pub outputs: BTreeMap<String, Vec<u8>>,
    public_outputs: BTreeSet<String>,
    public_media: BTreeSet<String>,
    budget: OutputBudget,
    modules: BTreeMap<PathBuf, Module>,
    visiting: BTreeSet<PathBuf>,
    assets: BTreeMap<PathBuf, String>,
    // ESM evaluates a CSS side-effect import once; CSS @import has occurrence
    // semantics and must repeat its rules at every position in the cascade.
    css_entries: BTreeSet<PathBuf>,
    css_cache: BTreeMap<PathBuf, String>,
    css_cache_bytes: usize,
    css_stack: BTreeSet<PathBuf>,
    css: String,
}

struct CssUrls<'a> {
    linker: &'a mut Linker,
    source: &'a Path,
}
impl<'i> style_compiler::style_optimizer::visitor::Visitor<'i> for CssUrls<'_> {
    type Error = AssetError;
    fn visit_types(&self) -> style_compiler::style_optimizer::visitor::VisitTypes {
        style_compiler::style_optimizer::visitor::VisitTypes::URLS
    }
    fn visit_url(
        &mut self,
        url: &mut style_compiler::style_optimizer::values::url::Url<'i>,
    ) -> Result<()> {
        let value = url.url.to_string();
        if value.starts_with('#') {
            return Ok(());
        }
        if value.starts_with("data:") {
            if self.linker.config.web.kind == OutputKind::Embedded {
                let encoded = value
                    .strip_prefix("data:image/svg+xml,")
                    .ok_or_else(|| AssetError::SvgDataUrl(self.source.into()))?;
                let svg = url_encoding::percent::percent_decode_str(encoded)
                    .decode_utf8()
                    .map_err(|source| AssetError::SvgEncoding {
                        path: self.source.into(),
                        source,
                    })?
                    .into_owned();
                let output = self.linker.output("blog-icon", "svg", svg.into_bytes())?;
                url.url = self.linker.css_url(&output).into();
            }
            return Ok(());
        }
        let specifier = if value.starts_with('.') || value.starts_with('/') {
            value
        } else {
            format!("./{value}")
        };
        let source = self
            .linker
            .resolve_with_public_priority(self.source, &specifier, true)?;
        let output = self.linker.asset(&source)?;
        url.url = self.linker.css_url(&output).into();
        Ok(())
    }
}

impl Linker {
    pub fn new(web: &Path, pkg: &Path, config: Config) -> Self {
        Self {
            web: web.into(),
            pkg: pkg.into(),
            config,
            outputs: BTreeMap::new(),
            public_outputs: BTreeSet::new(),
            public_media: BTreeSet::new(),
            budget: OutputBudget::default(),
            modules: BTreeMap::new(),
            visiting: BTreeSet::new(),
            assets: BTreeMap::new(),
            css_entries: BTreeSet::new(),
            css_cache: BTreeMap::new(),
            css_cache_bytes: 0,
            css_stack: BTreeSet::new(),
            css: String::new(),
        }
    }

    fn checked_source(&self, source: &Path) -> Result<PathBuf> {
        let source = source
            .strip_prefix(self.web.join("pkg"))
            .map(|relative| self.pkg.join(relative))
            .unwrap_or_else(|_| source.to_owned());
        let staged = self.pkg != self.web && source.starts_with(&self.pkg);
        if staged {
            inventory::relative(
                source
                    .strip_prefix(&self.pkg)
                    .expect("invariant: a staged source belongs to its package root"),
            )?;
        } else {
            let relative =
                source
                    .strip_prefix(&self.web)
                    .map_err(|error| AssetError::SourceEscape {
                        path: source.clone(),
                        source: error,
                    })?;
            source_inputs::web_relative(relative, false)
                .map_err(|error| input_error(&source, error))?;
        }
        inventory::physical(&source)?;
        let source = canonicalize(&source)?;
        let metadata = fs::metadata(&source).map_err(|error| AssetError::Inspect {
            path: source.clone(),
            source: error,
        })?;
        if !source.starts_with(if staged { &self.pkg } else { &self.web }) || !metadata.is_file() {
            return Err(AssetError::SourceFile(source));
        }
        Ok(source)
    }

    fn authored_source(&self, name: &str) -> Result<PathBuf> {
        source_inputs::web_relative(Path::new(name), false)
            .map_err(|source| input_error(Path::new(name), source))?;
        self.checked_source(&self.web.join(name))
    }

    fn resolve(&self, parent: &Path, specifier: &str) -> Result<PathBuf> {
        self.resolve_with_public_priority(parent, specifier, false)
    }

    fn resolve_with_public_priority(
        &self,
        parent: &Path,
        specifier: &str,
        public_first: bool,
    ) -> Result<PathBuf> {
        if specifier.contains(['\n', '\r', '\0', '\\', '#']) {
            return Err(AssetError::Import {
                specifier: specifier.into(),
                source: ImportError::Url,
            });
        }
        let mut parts = specifier.split('?');
        let plain = parts
            .next()
            .expect("invariant: split yields its first segment");
        if let Some(query) = parts.next()
            && (query != "url" || parts.next().is_some())
        {
            return Err(AssetError::Import {
                specifier: specifier.into(),
                source: ImportError::Query,
            });
        }
        let mut aliased = None;
        let mut aliases = self.config.aliases.iter().collect::<Vec<_>>();
        aliases.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
        for (name, destination) in aliases {
            if plain == name || name.ends_with('/') && plain.starts_with(name) {
                aliased = Some(if plain == name {
                    self.web.join(destination)
                } else {
                    self.web.join(destination).join(&plain[name.len()..])
                });
                break;
            }
        }
        let path = if let Some(path) = aliased {
            path
        } else if plain.starts_with('.') {
            parent
                .parent()
                .expect("invariant: a physical module file has a parent")
                .join(plain)
        } else if let Some(relative) = plain.strip_prefix('/') {
            // Public URLs are rooted at the output, not at a `public/` URL prefix.
            // CSS follows Vite's public-first URL lookup. ESM imports prefer an
            // actual Web source file, with public as a bounded fallback.
            inventory::relative(Path::new(relative))?;
            let web = self.web.join(relative);
            let public = self.web.join("public").join(relative);
            let (first, second) = if public_first {
                (public, web)
            } else {
                (web, public)
            };
            let mut candidate = first;
            let staged = |path: &Path| {
                path.strip_prefix(self.web.join("pkg"))
                    .map(|relative| self.pkg.join(relative))
                    .unwrap_or_else(|_| path.into())
            };
            let mut physical = staged(&candidate);
            let mut found = source_exists(&physical)?;
            if !found && candidate.extension().is_none() {
                candidate.set_extension("js");
                physical = staged(&candidate);
                found = source_exists(&physical)?;
            }
            // Do not conceal a dangling source symlink by falling back to public.
            if found { candidate } else { second }
        } else {
            return Err(AssetError::Import {
                specifier: specifier.into(),
                source: ImportError::Bare,
            });
        };
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if !normalized.pop() {
                        return Err(AssetError::Import {
                            specifier: specifier.into(),
                            source: ImportError::RootEscape,
                        });
                    }
                }
                std::path::Component::CurDir => {}
                component => normalized.push(component.as_os_str()),
            }
        }
        let mut path = if normalized.starts_with(self.web.join("pkg")) {
            self.pkg.join(
                normalized
                    .strip_prefix(self.web.join("pkg"))
                    .expect("invariant: a remapped source has the package prefix"),
            )
        } else {
            normalized
        };
        if path.extension().is_none() && !source_exists(&path)? {
            path.set_extension("js");
        }
        self.checked_source(&path)
            .map_err(|source| AssetError::Resolve {
                parent: parent.into(),
                specifier: specifier.into(),
                source: Box::new(source),
            })
    }

    fn generated_output(&mut self, name: String, bytes: Vec<u8>) -> Result<()> {
        if self.public_outputs.contains(&name) {
            return Err(AssetError::PublicCollision(name));
        }
        if let Some(previous) = self.outputs.get(&name) {
            if previous != &bytes {
                return Err(AssetError::GeneratedCollision(name));
            }
        } else {
            self.budget
                .add(&self.config.web, &name, bytes.len(), false)?;
            self.outputs.insert(name, bytes);
        }
        Ok(())
    }

    fn public_output(&mut self, source: &Path, bytes: Vec<u8>) -> Result<Option<String>> {
        let Ok(relative) = source.strip_prefix(self.web.join("public")) else {
            return Ok(None);
        };
        let name = inventory::relative(relative)?;
        if name == "index.html" {
            return Err(AssetError::PublicHtml);
        }
        if let Some(previous) = self.outputs.get(&name) {
            if !self.public_outputs.contains(&name) || previous != &bytes {
                return Err(AssetError::PublicCollision(name));
            }
        } else {
            self.budget.add(
                &self.config.web,
                &name,
                bytes.len(),
                self.public_media.contains(&name),
            )?;
            self.outputs.insert(name.clone(), bytes);
            self.public_outputs.insert(name.clone());
        }
        Ok(Some(name))
    }

    fn copy_public(&mut self) -> Result<()> {
        self.public_media.clear();
        let expected = if self.config.web.public_assets_manifest.is_empty() {
            None
        } else {
            let path = public_manifest_path(&self.web, &self.config.web.public_assets_manifest)?;
            let expected: PublicAssetsManifest =
                serde_json::from_slice(&inventory::read(&path)?)
                    .map_err(|source| AssetError::PublicManifest { path, source })?;
            if expected.format != 1 {
                return Err(AssetError::PublicManifestFormat(expected.format));
            }
            for (name, digest) in &expected.files {
                inventory::relative(Path::new(name))?;
                if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(AssetError::PublicDigest(name.clone()));
                }
            }
            self.public_media = expected
                .files
                .keys()
                .filter(|name| {
                    matches!(
                        mime_guess::from_path(name)
                            .first_or_octet_stream()
                            .type_()
                            .as_str(),
                        "image" | "video" | "audio"
                    )
                })
                .cloned()
                .collect();
            Some(expected.files)
        };
        let mut actual = Files::new();
        let public = self.web.join("public");
        for path in source_inputs::web_paths(&self.web, &public)
            .map_err(|source| input_error(&public, source))?
        {
            let bytes = inventory::read(&path)?;
            let digest = inventory::sha(&bytes);
            let name = inventory::relative(
                path.strip_prefix(self.web.join("public"))
                    .expect("invariant: an enumerated public path belongs to its root"),
            )?;
            if let Some(expected) = &expected
                && expected.get(&name) != Some(&digest)
            {
                return Err(AssetError::PublicInventory(
                    self.config.web.public_assets_manifest.clone(),
                ));
            }
            self.public_output(&path, bytes)?
                .expect("invariant: an enumerated public file belongs to the public root");
            actual.insert(name, digest);
        }
        if expected.is_some_and(|expected| actual != expected) {
            return Err(AssetError::PublicInventory(
                self.config.web.public_assets_manifest.clone(),
            ));
        }
        Ok(())
    }

    fn output(&mut self, stem: &str, extension: &str, bytes: Vec<u8>) -> Result<String> {
        let stem: String = stem
            .chars()
            .map(|v| {
                if v.is_ascii_alphanumeric() || v == '-' || v == '_' {
                    v
                } else {
                    '-'
                }
            })
            .collect();
        let name = format!(
            "assets/{stem}-{}.{}",
            &inventory::sha(&bytes)[..16],
            extension
        );
        self.generated_output(name.clone(), bytes)?;
        Ok(name)
    }

    fn url(&self, output: &str) -> String {
        format!("{}{}", self.config.web.base, output)
    }

    fn module_url(&self, output: &str) -> String {
        if self.config.web.relative_urls {
            if let Some(name) = output.strip_prefix("assets/") {
                format!("./{name}")
            } else {
                format!("../{output}")
            }
        } else {
            self.url(output)
        }
    }

    fn css_url(&self, output: &str) -> String {
        if self.config.web.relative_urls {
            let depth = Path::new(&self.config.web.css_output)
                .parent()
                .map(|p| p.components().count())
                .unwrap_or(1);
            format!("{}{output}", "../".repeat(depth))
        } else {
            self.url(output)
        }
    }

    fn asset(&mut self, source: &Path) -> Result<String> {
        let checked = self.checked_source(source)?;
        let source = checked.as_path();
        if let Some(name) = self.assets.get(source) {
            return Ok(name.clone());
        }
        let extension = source.extension().and_then(|v| v.to_str()).unwrap_or("bin");
        if extension == "map" {
            return Err(AssetError::SourceMap);
        }
        let bytes = inventory::read(source)?;
        if extension == "wasm" && !bytes.starts_with(b"\0asm") {
            return Err(AssetError::Wasm(source.into()));
        }
        let mut fixed = None;
        for (output, input) in &self.config.web.copies {
            if canonicalize(&self.web.join(input))? == source {
                fixed = Some(output.clone());
                break;
            }
        }
        let output = if source.starts_with(self.web.join("public")) {
            self.public_output(source, bytes)?
                .expect("invariant: a public source publishes with its relative name")
        } else if let Some(output) = fixed {
            self.generated_output(output.clone(), bytes)?;
            output
        } else {
            self.output(source_stem(source), extension, bytes)?
        };
        if extension == "wasm" {
            let mut compressed = GzBuilder::new()
                .mtime(0)
                .write(Vec::new(), Compression::best());
            compressed
                .write_all(&self.outputs[&output])
                .map_err(|error| AssetError::Compression {
                    path: source.into(),
                    source: error,
                })?;
            let compressed = compressed
                .finish()
                .map_err(|error| AssetError::Compression {
                    path: source.into(),
                    source: error,
                })?;
            self.generated_output(format!("{output}.gz"), compressed)?;
        }
        self.assets.insert(source.into(), output.clone());
        Ok(output)
    }

    fn stylesheet(&mut self, source: &Path) -> Result<String> {
        use style_compiler::style_optimizer::{
            rules::CssRule,
            stylesheet::{ParserOptions, PrinterOptions, StyleSheet},
            visitor::Visit,
        };
        let checked = self.checked_source(source)?;
        let source = checked.as_path();
        if self.css_stack.contains(source) {
            return Err(AssetError::CssCycle(source.into()));
        }
        if let Some(css) = self.css_cache.get(source) {
            return Ok(css.clone());
        }
        self.css_stack.insert(source.into());
        let text = utf8(inventory::read(source)?, source)?;
        let mut sheet = StyleSheet::parse(&text, ParserOptions::default()).map_err(|error| {
            AssetError::CssParse {
                path: source.into(),
                source: Box::new(owned_css_error(error)),
            }
        })?;
        let mut imported = String::new();
        for rule in &sheet.rules.0 {
            if let CssRule::Import(import) = rule {
                if !import.media.media_queries.is_empty()
                    || import.supports.is_some()
                    || import.layer.is_some()
                {
                    return Err(AssetError::ConditionalCss(source.into()));
                }
                let specifier = import.url.as_ref();
                let path = self.resolve(
                    source,
                    if specifier.starts_with('.') || specifier.starts_with('/') {
                        specifier.to_owned()
                    } else {
                        format!("./{specifier}")
                    }
                    .as_str(),
                )?;
                let css = self.stylesheet(&path)?;
                Self::append_css(&mut imported, &css, self.config.web.max_asset_bytes)?;
            }
        }
        sheet
            .rules
            .0
            .retain(|rule| !matches!(rule, CssRule::Import(_)));
        sheet.visit(&mut CssUrls {
            linker: self,
            source,
        })?;
        sheet
            .minify(style_compiler::style_optimizer::stylesheet::MinifyOptions::default())
            .map_err(|error| AssetError::CssMinify {
                path: source.into(),
                source: Box::new(error),
            })?;
        let code = sheet
            .to_css(PrinterOptions {
                minify: true,
                ..Default::default()
            })
            .map_err(|error| AssetError::CssPrint {
                path: source.into(),
                source: Box::new(error),
            })?
            .code;
        Self::append_css(&mut imported, &code, self.config.web.max_asset_bytes)?;
        self.css_stack.remove(source);
        // The cache is an optimization: a long import-only chain can have a
        // tiny final output but repeat the same expansion in every cache entry.
        // Decline new entries when full instead of rejecting a valid bundle.
        if let Some(size) = self.css_cache_bytes.checked_add(imported.len())
            && size <= self.config.web.max_total_bytes
        {
            self.css_cache.insert(source.into(), imported.clone());
            self.css_cache_bytes = size;
        }
        Ok(imported)
    }

    fn append_css(output: &mut String, css: &str, limit: usize) -> Result<()> {
        if output
            .len()
            .checked_add(css.len())
            .is_none_or(|size| size > limit)
        {
            return Err(BudgetError::Css { limit }.into());
        }
        output.push_str(css);
        Ok(())
    }

    pub fn module(&mut self, source: &Path) -> Result<String> {
        let source = self.checked_source(source)?;
        if let Some(module) = self.modules.get(&source) {
            return Ok(module.output.clone());
        }
        if !self.visiting.insert(source.clone()) {
            return Err(AssetError::ModuleCycle(source));
        }
        let text = utf8(inventory::read(&source)?, &source)?;
        let references =
            crate::javascript::references(&text).map_err(|error| AssetError::JavaScript {
                path: source.clone(),
                source: error,
            })?;
        let mut replacements = Vec::new();
        let mut children = BTreeSet::new();
        let mut derived_outputs = BTreeSet::new();
        for import in references.imports {
            let child = self.resolve(&source, &import.url)?;
            if child.extension().is_some_and(|v| v == "css") {
                if import.has_bindings {
                    return Err(AssetError::Import {
                        specifier: import.url,
                        source: ImportError::CssBindings,
                    });
                }
                if self.css_entries.insert(child.clone()) {
                    let css = self.stylesheet(&child)?;
                    if !self.css.is_empty() {
                        Self::append_css(&mut self.css, "\n", self.config.web.max_asset_bytes)?;
                    }
                    Self::append_css(&mut self.css, &css, self.config.web.max_asset_bytes)?;
                }
                let (start, end) = crate::javascript::range(import.statement);
                replacements.push((start, end, String::new()));
            } else if import.url.ends_with("?url") {
                if !import.default_asset {
                    return Err(AssetError::Import {
                        specifier: import.url,
                        source: ImportError::DefaultAsset,
                    });
                }
                let output = self.asset(&child)?;
                let value = js_string(&self.module_url(&output));
                let expression = if self.config.web.relative_urls {
                    format!("new URL({value},import.meta.url).href")
                } else {
                    value
                };
                // Keep import/export declarations intact. A separate ESM owns
                // the URL value, so exported names need not be identifiers and
                // the consuming module cannot capture the generated URL call.
                let bytes = crate::javascript::minify(&format!("export default {expression};"))
                    .map_err(|error| AssetError::JavaScript {
                        path: source.clone(),
                        source: error,
                    })?;
                let module = self.output("asset-url", "js", bytes)?;
                derived_outputs.insert(module.clone());
                let (start, end) = crate::javascript::range(import.source);
                replacements.push((start, end, js_string(&self.module_url(&module))));
            } else {
                if !matches!(
                    child.extension().and_then(|value| value.to_str()),
                    Some("js" | "mjs")
                ) {
                    return Err(AssetError::Import {
                        specifier: import.url,
                        source: ImportError::ModuleExtension,
                    });
                }
                let output = self.module(&child)?;
                children.insert(child);
                let (start, end) = crate::javascript::range(import.source);
                replacements.push((start, end, js_string(&self.module_url(&output))));
            }
        }
        for (url, span) in references.assets {
            let specifier = if url.starts_with('.') || url.starts_with('/') {
                url
            } else {
                format!("./{url}")
            };
            let child = self.resolve(&source, &specifier)?;
            let output = self.asset(&child)?;
            let (start, end) = crate::javascript::range(span);
            replacements.push((start, end, js_string(&self.module_url(&output))));
        }
        for (url, span) in references.dynamic {
            let child = self.resolve(&source, &url)?;
            let output = self.module(&child)?;
            children.insert(child);
            let (start, end) = crate::javascript::range(span);
            replacements.push((start, end, js_string(&self.module_url(&output))));
        }
        replacements.sort_by_key(|(start, _, _)| *start);
        let mut transformed = String::new();
        let mut end = 0;
        for (start, finish, replacement) in replacements {
            assert!(
                start >= end,
                "invariant: AST references occupy disjoint source ranges"
            );
            transformed.push_str(&text[end..start]);
            transformed.push_str(&replacement);
            end = finish;
        }
        transformed.push_str(&text[end..]);
        let bytes =
            crate::javascript::minify(&transformed).map_err(|error| AssetError::JavaScript {
                path: source.clone(),
                source: error,
            })?;
        let output = self.output(source_stem(&source), "js", bytes)?;
        self.modules.insert(
            source.clone(),
            Module {
                source: source.clone(),
                output: output.clone(),
                imports: children,
                derived_outputs: derived_outputs.into_iter().collect(),
            },
        );
        self.visiting.remove(&source);
        Ok(output)
    }

    pub fn finish(mut self, provenance: &[u8]) -> Result<(BTreeMap<String, Vec<u8>>, Value)> {
        self.copy_public()?;
        let mut script = String::new();
        if self.config.web.kind == OutputKind::Embedded {
            for (facade, input) in std::mem::take(&mut self.config.web.entries) {
                let source = self.authored_source(&input)?;
                let output = self.module(&source)?;
                let depth = Path::new(&facade)
                    .parent()
                    .map(|p| p.components().count())
                    .unwrap_or(0);
                let target = format!("{}{output}", "../".repeat(depth));
                let wrapper = crate::javascript::minify(&format!(
                    "export * from {};",
                    js_string(&if target.starts_with('.') {
                        target
                    } else {
                        format!("./{target}")
                    })
                ))
                .map_err(|error| AssetError::JavaScript {
                    path: source.clone(),
                    source: error,
                })?;
                self.generated_output(facade.clone(), wrapper)?;
                self.modules
                    .get_mut(&source)
                    .expect("invariant: a linked entry is cached as a module")
                    .derived_outputs
                    .push(facade);
            }
        } else {
            let entry = self.web.join(&self.config.web.entry);
            script = self.module(&entry)?;
        }
        if !self.outputs.keys().any(|name| name.ends_with(".wasm")) {
            return Err(AssetError::MissingWasm);
        }
        let css_bytes = std::mem::take(&mut self.css).into_bytes();
        let css = if self.config.web.css_output.is_empty() {
            self.output("style", "css", css_bytes)?
        } else {
            let name = self.config.web.css_output.clone();
            self.generated_output(name.clone(), css_bytes)?;
            name
        };
        let copies = std::mem::take(&mut self.config.web.copies);
        for (output, input) in &copies {
            let source = self.authored_source(input)?;
            let bytes = inventory::read(&source)?;
            self.generated_output(output.clone(), bytes)?;
        }
        self.config.web.copies = copies;
        for license in std::mem::take(&mut self.config.web.licenses) {
            let path = self.authored_source(&license)?;
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .expect("invariant: a validated license source has a UTF-8 filename");
            let output = format!("assets/{name}");
            let bytes = inventory::read(&path)?;
            if bytes.is_empty() {
                return Err(AssetError::EmptyLicense(path));
            }
            self.generated_output(output, bytes)?;
        }
        self.generated_output("assets/blog-assets.json".into(), provenance.into())?;
        let repository = self
            .web
            .ancestors()
            .nth(Path::new(layout::WEB).components().count())
            .ok_or_else(|| AssetError::RepositoryRoot(self.web.clone()))?;
        let vendor_sources = inventory::walk(&repository.join(layout::INTERFACE))?
            .into_iter()
            .chain(inventory::walk(&self.web.join("styles"))?)
            .filter(|p| p.file_name().is_some_and(|v| v == "ORIGIN.json"))
            .map(|p| {
                Ok((
                    inventory::relative(p.strip_prefix(repository).expect(
                        "invariant: a source walked under the repository retains its prefix",
                    ))?,
                    serde_json::from_slice::<Value>(&inventory::read(&p)?)
                        .map_err(|source| AssetError::VendorSource { path: p, source })?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        self.generated_output(
            "assets/vendor-sources.json".into(),
            serde_json::to_vec_pretty(&vendor_sources)
                .expect("invariant: vendor source JSON values are serializable"),
        )?;
        if self.config.web.kind == OutputKind::Spa {
            let index = self.web.join("index.html");
            let mut html = utf8(inventory::read(&index)?, &index)?;
            let script_tag = Regex::new(r#"<script\s+[^>]*type=["']module["'][^>]*>\s*</script>"#)
                .expect("invariant: the static module-script regex is valid");
            if script_tag.find_iter(&html).count() != 1 {
                return Err(AssetError::HtmlEntry);
            }
            html = script_tag
                .replace(
                    &html,
                    format!(
                        "<script type=\"module\" src=\"{}\"></script>",
                        self.url(&script)
                    ),
                )
                .into_owned();
            let node_fonts = Regex::new(r#"(?:\./|/)?styles/web-fonts/([A-Za-z0-9_.-]+)"#)
                .expect("invariant: the static font-preload regex is valid");
            let mut rewritten = String::new();
            let mut end = 0;
            for capture in node_fonts.captures_iter(&html) {
                let found = capture
                    .get(0)
                    .expect("invariant: a regex capture contains the full match");
                rewritten.push_str(&html[end..found.start()]);
                let path = self.web.join("styles/web-fonts").join(&capture[1]);
                let output = self.asset(&path)?;
                rewritten.push_str(&self.url(&output));
                end = found.end();
            }
            rewritten.push_str(&html[end..]);
            html = rewritten;
            if html.contains("node_modules") || html.contains("/src/") {
                return Err(AssetError::HtmlDevelopmentReference);
            }
            let mut preload = format!("<link rel=\"stylesheet\" href=\"{}\">\n", self.url(&css));
            for name in self
                .outputs
                .keys()
                .filter(|name| name.ends_with(".js") && **name != script)
            {
                preload.push_str(&format!(
                    "<link rel=\"modulepreload\" href=\"{}\">\n",
                    self.url(name)
                ));
            }
            for name in self.outputs.keys().filter(|name| name.ends_with(".wasm")) {
                preload.push_str(&format!("<link rel=\"preload\" href=\"{}\" as=\"fetch\" type=\"application/wasm\" crossorigin=\"anonymous\">\n", self.url(name)));
            }
            if !html.contains("</head>") {
                return Err(AssetError::HtmlHead);
            }
            html = html.replacen("</head>", &format!("{preload}</head>"), 1);
            self.generated_output("index.html".into(), html.into_bytes())?;
        }
        self.budgets()?;
        let ordered = self.modules.keys().cloned().collect::<Vec<_>>();
        let ids = ordered
            .iter()
            .enumerate()
            .map(|(id, path)| (path.clone(), id))
            .collect::<BTreeMap<_, _>>();
        let mut modules = Vec::new();
        for path in ordered {
            let module = &self.modules[&path];
            let relative = if module.source.starts_with(&self.pkg) {
                format!(
                    "pkg/{}",
                    inventory::relative(
                        module
                            .source
                            .strip_prefix(&self.pkg)
                            .expect("invariant: a package module retains its source prefix")
                    )?
                )
            } else {
                inventory::relative(
                    module
                        .source
                        .strip_prefix(&self.web)
                        .expect("invariant: a linked module belongs to the web root"),
                )?
            };
            let relative = if self.config.web.runtime_root == RuntimeRoot::Product {
                format!("{}/{relative}", layout::WEB)
            } else {
                relative
            };
            let mut delivered = vec![module.output.clone()];
            delivered.extend(module.derived_outputs.clone());
            modules.push(json!({"path":relative,"sha256":inventory::sha(&inventory::read(&module.source)?),"files":delivered,"imports":module.imports.iter().map(|path|ids[path]).collect::<Vec<_>>()}));
        }
        let outputs = self
            .outputs
            .iter()
            .map(|(name, bytes)| json!({"path":name,"sha256":inventory::sha(bytes)}))
            .collect::<Vec<_>>();
        Ok((
            self.outputs,
            json!({"format_version":1,"distribution":"dist","modules":modules,"outputs":outputs}),
        ))
    }

    fn budgets(&self) -> Result<()> {
        let mut budget = OutputBudget::default();
        for (name, bytes) in &self.outputs {
            budget.add(
                &self.config.web,
                name,
                bytes.len(),
                self.public_media.contains(name),
            )?;
        }
        Ok(())
    }
}

pub fn write_outputs(root: &Path, outputs: &BTreeMap<String, Vec<u8>>) -> Result<Files> {
    for (name, bytes) in outputs {
        inventory::relative(Path::new(name))?;
        let path = root.join(name);
        let parent = path
            .parent()
            .expect("invariant: a validated output has a parent directory");
        fs::create_dir_all(parent).map_err(|source| AssetError::CreateOutputDirectory {
            path: parent.into(),
            source,
        })?;
        fs::write(&path, bytes).map_err(|source| AssetError::WriteOutput { path, source })?;
    }
    Ok(inventory::files(root)?)
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/assets.rs"]
mod tests;
