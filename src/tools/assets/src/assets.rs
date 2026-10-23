//! A deliberately bounded ESM linker: no transpilation, eval, npm install or network.
//! Only static ESM imports, CSS imports and literal import.meta.url assets are accepted.
use crate::inventory::{self, Files};
use anyhow::{Context, Result, bail, ensure};
use compression::streams::{Compression, GzBuilder};
use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const MAX_PUBLIC_MEDIA_BYTES: usize = 4 * 1024 * 1024;
const MAX_PUBLIC_MEDIA_TOTAL_BYTES: usize = 8 * 1024 * 1024;

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
    ensure!(
        name.components().count() == 1 && name.extension().is_some_and(|ext| ext == "json"),
        "public_assets_manifest must name a JSON file directly in src/web/"
    );
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

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Web {
    pub kind: String,
    pub entry: String,
    pub entries: BTreeMap<String, String>,
    pub copies: BTreeMap<String, String>,
    pub css_output: String,
    pub relative_urls: bool,
    pub runtime_root: String,
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
            kind: "spa".into(),
            entry: String::new(),
            entries: BTreeMap::new(),
            copies: BTreeMap::new(),
            css_output: String::new(),
            relative_urls: false,
            runtime_root: "web".into(),
            dist: "src/web/dist".into(),
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
        let mut config: Self = if path.exists() {
            toml::from_str(&String::from_utf8(inventory::read(&path)?)?)?
        } else {
            Self::default()
        };
        if config.web.entry.is_empty() {
            config.web.entry = if web.join("src/main-blog-browser.js").is_file() {
                "src/main-blog-browser.js"
            } else {
                "src/main.js"
            }
            .into();
        }
        inventory::relative(Path::new(&config.web.entry))?;
        ensure!(
            matches!(config.web.kind.as_str(), "spa" | "embedded"),
            "web.kind must be spa or embedded"
        );
        ensure!(
            matches!(config.web.runtime_root.as_str(), "web" | "product"),
            "web.runtime_root must be web or product"
        );
        if config.web.kind == "embedded" {
            ensure!(
                !config.web.entries.is_empty(),
                "Embedded output needs explicit entry points"
            );
        }
        for (output, source) in config.web.entries.iter().chain(config.web.copies.iter()) {
            inventory::relative(Path::new(output))?;
            inventory::relative(Path::new(source))?;
        }
        if !config.web.css_output.is_empty() {
            inventory::relative(Path::new(&config.web.css_output))?;
        }
        if !config.web.public_assets_manifest.is_empty() {
            public_manifest_path(web, &config.web.public_assets_manifest)?;
        }
        ensure!(
            config.web.base.starts_with('/')
                && config.web.base.ends_with('/')
                && !config
                    .web
                    .base
                    .contains(['\\', '?', '#', '"', '\'', '<', '>', '\n', '\r'])
                && !config.web.base.contains("..")
                && !config.web.base.starts_with("//"),
            "base must be a local /path/ URL"
        );
        for limit in [
            config.web.max_asset_bytes,
            config.web.max_wasm_bytes,
            config.web.max_font_bytes,
            config.web.max_total_bytes,
        ] {
            ensure!(
                (1..=64 * 1024 * 1024).contains(&limit),
                "Asset budget must be in 1..67108864"
            );
        }
        // Raising an established project budget requires a separately reviewed code change.
        ensure!(
            config.web.max_asset_bytes <= 512 * 1024
                && config.web.max_wasm_bytes <= 4 * 1024 * 1024
                && config.web.max_font_bytes <= 16 * 1024 * 1024
                && config.web.max_total_bytes <= 16 * 1024 * 1024,
            "Configuration may tighten but cannot relax asset budgets"
        );
        for destination in config.aliases.values() {
            inventory::relative(Path::new(destination.trim_end_matches('/')))?;
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
    modules: BTreeMap<PathBuf, Module>,
    visiting: BTreeSet<PathBuf>,
    assets: BTreeMap<PathBuf, String>,
    // ESM evaluates a CSS side-effect import once; CSS @import has occurrence
    // semantics and must repeat its rules at every position in the cascade.
    css_entries: BTreeSet<PathBuf>,
    css_cache: BTreeMap<PathBuf, String>,
    css_cache_bytes: usize,
    css_stack: BTreeSet<PathBuf>,
    css: Vec<String>,
}

struct CssUrls<'a> {
    linker: &'a mut Linker,
    source: &'a Path,
}
impl<'i> style_compiler::style_optimizer::visitor::Visitor<'i> for CssUrls<'_> {
    type Error = anyhow::Error;
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
            if self.linker.config.web.kind == "embedded" {
                let encoded=value.strip_prefix("data:image/svg+xml,").context("Embedded CSP only supports authored SVG data URLs, emitted as same-origin files")?;
                let svg = url_encoding::percent::percent_decode_str(encoded)
                    .decode_utf8()?
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
            modules: BTreeMap::new(),
            visiting: BTreeSet::new(),
            assets: BTreeMap::new(),
            css_entries: BTreeSet::new(),
            css_cache: BTreeMap::new(),
            css_cache_bytes: 0,
            css_stack: BTreeSet::new(),
            css: Vec::new(),
        }
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
        ensure!(
            !specifier.contains(['\n', '\r', '\0', '\\', '#']),
            "Unsupported module/resource URL {specifier:?}"
        );
        let mut parts = specifier.split('?');
        let plain = parts.next().unwrap_or_default();
        if let Some(query) = parts.next() {
            ensure!(
                query == "url" && parts.next().is_none(),
                "Only ?url asset imports are supported"
            );
        }
        let mut aliased = None;
        let mut aliases = self.config.aliases.iter().collect::<Vec<_>>();
        aliases.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
        for (name, destination) in aliases {
            if plain == name || name.ends_with('/') && plain.starts_with(name) {
                aliased = Some(if plain == name {
                    self.web.join(destination)
                } else {
                    self.web
                        .join(destination)
                        .join(plain.strip_prefix(name).unwrap_or_default())
                });
                break;
            }
        }
        let path = if let Some(path) = aliased {
            path
        } else if plain.starts_with('.') {
            parent
                .parent()
                .context("Missing module parent")?
                .join(plain)
        } else if plain.starts_with('/') {
            // Public URLs are rooted at the output, not at a `public/` URL prefix.
            // CSS follows Vite's public-first URL lookup. ESM imports prefer an
            // actual Web source file, with public as a bounded fallback.
            let relative = plain.strip_prefix('/').context("Absolute resource URL")?;
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
            if !physical.try_exists()?
                && fs::symlink_metadata(&physical).is_err()
                && candidate.extension().is_none()
            {
                candidate.set_extension("js");
                physical = staged(&candidate);
            }
            // Do not conceal a dangling source symlink by falling back to public.
            if physical.try_exists()? || fs::symlink_metadata(&physical).is_ok() {
                candidate
            } else {
                second
            }
        } else {
            bail!("Unvendored bare import {specifier:?}; configure an explicit alias");
        };
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    ensure!(normalized.pop(), "Import path escapes filesystem root");
                }
                std::path::Component::CurDir => {}
                component => normalized.push(component.as_os_str()),
            }
        }
        let mut path = if normalized.starts_with(self.web.join("pkg")) {
            self.pkg
                .join(normalized.strip_prefix(self.web.join("pkg"))?)
        } else {
            normalized
        };
        if !path.exists() && path.extension().is_none() {
            path.set_extension("js");
        }
        inventory::physical(&path)?;
        let path = path
            .canonicalize()
            .with_context(|| format!("Resolve {specifier:?} in {}", parent.display()))?;
        ensure!(
            path.starts_with(&self.web) || path.starts_with(&self.pkg),
            "Import escapes the Web source workspace"
        );
        ensure!(path.is_file(), "Import must refer to a physical file");
        Ok(path)
    }

    fn generated_output(&mut self, name: String, bytes: Vec<u8>) -> Result<()> {
        ensure!(
            !self.public_outputs.contains(&name),
            "Public asset shadows a generated resource: {name}"
        );
        if let Some(previous) = self.outputs.get(&name) {
            ensure!(previous == &bytes, "Conflicting generated output: {name}");
        } else {
            self.outputs.insert(name, bytes);
        }
        Ok(())
    }

    fn public_output(&mut self, source: &Path, bytes: Vec<u8>) -> Result<Option<String>> {
        let Ok(relative) = source.strip_prefix(self.web.join("public")) else {
            return Ok(None);
        };
        let name = inventory::relative(relative)?;
        ensure!(name != "index.html", "Public asset shadows generated HTML");
        if let Some(previous) = self.outputs.get(&name) {
            ensure!(
                self.public_outputs.contains(&name) && previous == &bytes,
                "Public asset shadows a generated resource: {name}"
            );
        } else {
            self.outputs.insert(name.clone(), bytes);
            self.public_outputs.insert(name.clone());
        }
        Ok(Some(name))
    }

    fn copy_public(&mut self) -> Result<()> {
        self.public_media.clear();
        let mut actual = Files::new();
        for path in inventory::walk(&self.web.join("public"))? {
            let bytes = inventory::read(&path)?;
            let digest = inventory::sha(&bytes);
            let name = self
                .public_output(&path, bytes)?
                .context("Public source path")?;
            actual.insert(name, digest);
        }
        if !self.config.web.public_assets_manifest.is_empty() {
            let path = public_manifest_path(&self.web, &self.config.web.public_assets_manifest)?;
            let expected: PublicAssetsManifest = serde_json::from_slice(&inventory::read(&path)?)?;
            ensure!(
                expected.format == 1,
                "Unsupported public assets manifest format"
            );
            for (name, digest) in &expected.files {
                inventory::relative(Path::new(name))?;
                ensure!(
                    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
                    "Invalid public asset SHA-256: {name}"
                );
            }
            ensure!(
                actual == expected.files,
                "Public assets differ from source inventory: {}",
                path.display()
            );
            self.public_media = actual
                .into_keys()
                .filter(|name| {
                    matches!(
                        mime_guess::from_path(name)
                            .first_or_octet_stream()
                            .type_()
                            .as_str(),
                        "image" | "video" | "audio"
                    )
                })
                .collect();
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
        if let Some(name) = self.assets.get(source) {
            return Ok(name.clone());
        }
        let extension = source.extension().and_then(|v| v.to_str()).unwrap_or("bin");
        ensure!(extension != "map", "Source maps cannot be published");
        let bytes = inventory::read(source)?;
        if extension == "wasm" {
            ensure!(bytes.starts_with(b"\0asm"), "Expected WebAssembly bytes");
        }
        let fixed = self.config.web.copies.iter().find_map(|(output, input)| {
            self.web
                .join(input)
                .canonicalize()
                .ok()
                .filter(|path| path == source)
                .map(|_| output.clone())
        });
        let output = if let Some(output) = self.public_output(source, bytes.clone())? {
            output
        } else if let Some(output) = fixed {
            self.generated_output(output.clone(), bytes.clone())?;
            output
        } else {
            self.output(
                source
                    .file_stem()
                    .and_then(|v| v.to_str())
                    .unwrap_or("asset"),
                extension,
                bytes.clone(),
            )?
        };
        if extension == "wasm" {
            let mut compressed = GzBuilder::new()
                .mtime(0)
                .write(Vec::new(), Compression::best());
            compressed.write_all(&bytes)?;
            self.generated_output(format!("{output}.gz"), compressed.finish()?)?;
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
        ensure!(
            !self.css_stack.contains(source),
            "Circular CSS @import: {}",
            source.display()
        );
        if let Some(css) = self.css_cache.get(source) {
            return Ok(css.clone());
        }
        self.css_stack.insert(source.into());
        let text = String::from_utf8(inventory::read(source)?)?;
        let mut sheet = StyleSheet::parse(&text, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS syntax error in {}: {e}", source.display()))?;
        let mut imported = String::new();
        for rule in &sheet.rules.0 {
            if let CssRule::Import(import) = rule {
                ensure!(
                    import.media.media_queries.is_empty()
                        && import.supports.is_none()
                        && import.layer.is_none(),
                    "Conditional CSS imports must be explicitly bundled upstream"
                );
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
                self.append_css(&mut imported, &css)?;
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
            .map_err(|e| anyhow::anyhow!("CSS minify error: {e}"))?;
        let code = sheet
            .to_css(PrinterOptions {
                minify: true,
                ..Default::default()
            })
            .map_err(|e| anyhow::anyhow!("CSS print error: {e}"))?
            .code;
        self.append_css(&mut imported, &code)?;
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

    fn append_css(&self, output: &mut String, css: &str) -> Result<()> {
        ensure!(
            output
                .len()
                .checked_add(css.len())
                .is_some_and(|size| { size <= self.config.web.max_asset_bytes }),
            "Expanded CSS exceeds {}-byte asset budget",
            self.config.web.max_asset_bytes
        );
        output.push_str(css);
        Ok(())
    }

    pub fn module(&mut self, source: &Path) -> Result<String> {
        let source = source.canonicalize()?;
        if let Some(module) = self.modules.get(&source) {
            return Ok(module.output.clone());
        }
        ensure!(
            self.visiting.insert(source.clone()),
            "Circular ESM imports require a reviewed prebundled ESM distribution: {}",
            source.display()
        );
        let text = String::from_utf8(inventory::read(&source)?)?;
        let references = crate::javascript::references(&text)
            .with_context(|| format!("Parse {}", source.display()))?;
        let mut replacements = Vec::new();
        let mut children = BTreeSet::new();
        let mut derived_outputs = BTreeSet::new();
        for import in references.imports {
            let child = self.resolve(&source, &import.url)?;
            if child.extension().is_some_and(|v| v == "css") {
                ensure!(
                    !import.has_bindings,
                    "CSS imports cannot create JavaScript bindings"
                );
                if self.css_entries.insert(child.clone()) {
                    let css = self.stylesheet(&child)?;
                    self.css.push(css);
                }
                let (start, end) = crate::javascript::range(import.statement);
                replacements.push((start, end, String::new()));
            } else if import.url.ends_with("?url") {
                ensure!(
                    import.default_asset,
                    "?url requires a default import or export {{default as name}}"
                );
                let output = self.asset(&child)?;
                let value = serde_json::to_string(&self.module_url(&output))?;
                let expression = if self.config.web.relative_urls {
                    format!("new URL({value},import.meta.url).href")
                } else {
                    value
                };
                // Keep import/export declarations intact. A separate ESM owns
                // the URL value, so exported names need not be identifiers and
                // the consuming module cannot capture the generated URL call.
                let bytes = crate::javascript::minify(&format!("export default {expression};"))?;
                let module = self.output("asset-url", "js", bytes)?;
                derived_outputs.insert(module.clone());
                let (start, end) = crate::javascript::range(import.source);
                replacements.push((
                    start,
                    end,
                    serde_json::to_string(&self.module_url(&module))?,
                ));
            } else {
                ensure!(
                    matches!(
                        child.extension().and_then(|v| v.to_str()),
                        Some("js" | "mjs")
                    ),
                    "ESM module must be .js or .mjs"
                );
                let output = self.module(&child)?;
                children.insert(child);
                let (start, end) = crate::javascript::range(import.source);
                replacements.push((
                    start,
                    end,
                    serde_json::to_string(&self.module_url(&output))?,
                ));
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
            replacements.push((
                start,
                end,
                serde_json::to_string(&self.module_url(&output))?,
            ));
        }
        for (url, span) in references.dynamic {
            let child = self.resolve(&source, &url)?;
            let output = self.module(&child)?;
            children.insert(child);
            let (start, end) = crate::javascript::range(span);
            replacements.push((
                start,
                end,
                serde_json::to_string(&self.module_url(&output))?,
            ));
        }
        replacements.sort_by_key(|(start, _, _)| *start);
        let mut transformed = String::new();
        let mut end = 0;
        for (start, finish, replacement) in replacements {
            ensure!(start >= end, "Ambiguous overlapping AST references");
            transformed.push_str(&text[end..start]);
            transformed.push_str(&replacement);
            end = finish;
        }
        transformed.push_str(&text[end..]);
        let bytes = crate::javascript::minify(&transformed)?;
        let output = self.output(
            source
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("module"),
            "js",
            bytes,
        )?;
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
        if self.config.web.kind == "embedded" {
            for (facade, input) in self.config.web.entries.clone() {
                let source = self.web.join(input).canonicalize()?;
                let output = self.module(&source)?;
                let depth = Path::new(&facade)
                    .parent()
                    .map(|p| p.components().count())
                    .unwrap_or(0);
                let target = format!("{}{output}", "../".repeat(depth));
                let wrapper = crate::javascript::minify(&format!(
                    "export * from {};",
                    serde_json::to_string(&if target.starts_with('.') {
                        target
                    } else {
                        format!("./{target}")
                    })?
                ))?;
                self.generated_output(facade.clone(), wrapper)?;
                self.modules
                    .get_mut(&source)
                    .context("Entry module")?
                    .derived_outputs
                    .push(facade);
            }
        } else {
            let entry = self.web.join(&self.config.web.entry);
            script = self.module(&entry)?;
        }
        ensure!(
            self.outputs.keys().any(|name| name.ends_with(".wasm")),
            "No independent WASM output was linked"
        );
        let css_bytes = self.css.join("\n").into_bytes();
        let css = if self.config.web.css_output.is_empty() {
            self.output("style", "css", css_bytes)?
        } else {
            let name = self.config.web.css_output.clone();
            self.generated_output(name.clone(), css_bytes)?;
            name
        };
        for (output, input) in self.config.web.copies.clone() {
            let source = self.web.join(input);
            let bytes = inventory::read(&source)?;
            self.generated_output(output, bytes)?;
        }
        for license in self.config.web.licenses.clone() {
            let path = self.web.join(&license);
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .context("License filename")?;
            let output = format!("assets/{name}");
            let bytes = inventory::read(&path)?;
            ensure!(!bytes.is_empty(), "Required license is empty");
            self.generated_output(output, bytes)?;
        }
        self.generated_output("assets/blog-assets.json".into(), provenance.into())?;
        let repository = self
            .web
            .parent()
            .and_then(Path::parent)
            .context("Asset source repository")?;
        let vendor_sources = inventory::walk(&repository.join("src/interface"))?
            .into_iter()
            .chain(inventory::walk(&self.web.join("styles"))?)
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|p| p.file_name().is_some_and(|v| v == "ORIGIN.json"))
            .map(|p| {
                Ok((
                    inventory::relative(p.strip_prefix(repository)?)?,
                    serde_json::from_slice::<Value>(&inventory::read(&p)?)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        self.generated_output(
            "assets/vendor-sources.json".into(),
            serde_json::to_vec_pretty(&vendor_sources)?,
        )?;
        if self.config.web.kind == "spa" {
            let mut html = String::from_utf8(inventory::read(&self.web.join("index.html"))?)?;
            let script_tag = Regex::new(r#"<script\s+[^>]*type=["']module["'][^>]*>\s*</script>"#)?;
            ensure!(
                script_tag.find_iter(&html).count() == 1,
                "index.html must have exactly one external module entry"
            );
            html = script_tag
                .replace(
                    &html,
                    format!(
                        "<script type=\"module\" src=\"{}\"></script>",
                        self.url(&script)
                    ),
                )
                .into_owned();
            let node_fonts = Regex::new(r#"(?:\./|/)?styles/web-fonts/([A-Za-z0-9_.-]+)"#)?;
            let mut rewritten = String::new();
            let mut end = 0;
            for capture in node_fonts.captures_iter(&html) {
                let found = capture.get(0).context("Font preload")?;
                rewritten.push_str(&html[end..found.start()]);
                let path = self.web.join("styles/web-fonts").join(&capture[1]);
                let output = self.asset(&path)?;
                rewritten.push_str(&self.url(&output));
                end = found.end();
            }
            rewritten.push_str(&html[end..]);
            html = rewritten;
            ensure!(
                !html.contains("node_modules") && !html.contains("/src/"),
                "HTML still contains a development-only resource reference"
            );
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
            ensure!(html.contains("</head>"), "HTML is missing head terminator");
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
                    inventory::relative(module.source.strip_prefix(&self.pkg)?)?
                )
            } else {
                inventory::relative(module.source.strip_prefix(&self.web)?)?
            };
            let relative = if self.config.web.runtime_root == "product" {
                format!("src/web/{relative}")
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
        let mut bundle = 0;
        let mut media = 0;
        for (name, bytes) in &self.outputs {
            ensure!(!name.ends_with(".map"), "Source maps cannot be published");
            if self.public_media.contains(name) {
                ensure!(
                    bytes.len() <= MAX_PUBLIC_MEDIA_BYTES,
                    "Public media {name} exceeds {MAX_PUBLIC_MEDIA_BYTES}-byte budget ({} bytes)",
                    bytes.len()
                );
                media += bytes.len();
                continue;
            }
            let limit = if name.ends_with(".woff") || name.ends_with(".woff2") {
                self.config.web.max_font_bytes
            } else if name.ends_with(".wasm") || name.ends_with(".wasm.gz") {
                self.config.web.max_wasm_bytes
            } else {
                self.config.web.max_asset_bytes
            };
            ensure!(
                bytes.len() <= limit,
                "Asset {name} exceeds {limit}-byte budget ({} bytes)",
                bytes.len()
            );
            bundle += bytes.len();
        }
        ensure!(
            bundle <= self.config.web.max_total_bytes,
            "Web bundle exceeds {}-byte total budget ({bundle} bytes)",
            self.config.web.max_total_bytes
        );
        ensure!(
            media <= MAX_PUBLIC_MEDIA_TOTAL_BYTES,
            "Public media exceed {MAX_PUBLIC_MEDIA_TOTAL_BYTES}-byte total budget ({media} bytes)"
        );
        Ok(())
    }
}

pub fn write_outputs(root: &Path, outputs: &BTreeMap<String, Vec<u8>>) -> Result<Files> {
    for (name, bytes) in outputs {
        inventory::relative(Path::new(name))?;
        let path = root.join(name);
        fs::create_dir_all(path.parent().context("Output parent")?)?;
        fs::write(path, bytes)?;
    }
    inventory::files(root)
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/assets.rs"]
mod tests;
