use super::*;
use anyhow::Result;
use std::error::Error;

fn diagnostic(error: &AssetError) -> String {
    let mut messages = Vec::new();
    let mut current = Some(error as &(dyn Error + 'static));
    while let Some(error) = current {
        messages.push(error.to_string());
        current = error.source();
    }
    messages.join(": ")
}

#[test]
fn output_modes_are_parsed_at_the_configuration_boundary() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rust-web.toml");
    for (field, unknown) in [("kind", "server"), ("runtime_root", "external")] {
        fs::write(&path, format!("[web]\n{field}={unknown:?}\n"))?;
        let error = Config::read(directory.path()).unwrap_err();
        assert!(error.source().is_some());
        let AssetError::Configuration { source, .. } = error else {
            panic!("expected a TOML configuration failure");
        };
        assert!(!source.message().is_empty());
    }
    fs::write(
        path,
        "[web]\nkind='embedded'\nruntime_root='product'\n[web.entries]\n'entry.js'='src/main.js'\n",
    )?;
    let config = Config::read(directory.path())?;
    assert_eq!(config.web.kind, OutputKind::Embedded);
    assert_eq!(config.web.runtime_root, RuntimeRoot::Product);
    Ok(())
}

#[test]
fn css_compiler_keeps_values_while_minifying_and_rejects_invalid_colors() {
    use style_compiler::style_optimizer::{
        printer::PrinterOptions,
        stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    };
    let mut sheet = StyleSheet::parse(
        r#".box{color:RGB(255,0,0);margin:0px 0px 0px 0px;content:"a  b";background:url("./keep.svg")}"#,
        ParserOptions::default(),
    )
    .unwrap();
    sheet.minify(MinifyOptions::default()).unwrap();
    let code = sheet
        .to_css(PrinterOptions {
            minify: true,
            ..PrinterOptions::default()
        })
        .unwrap()
        .code;
    assert!(code.contains("color:red"), "{code}");
    assert!(code.contains("margin:0"), "{code}");
    assert!(code.contains("a  b"), "{code}");
    assert!(code.contains("./keep.svg"), "{code}");
    StyleSheet::parse(&code, ParserOptions::default()).unwrap();
    // Stylesheets preserve unparsed declarations; strict value parsing rejects invalid colors.
    use style_compiler::style_optimizer::{traits::Parse, values::color::CssColor};
    let mut input = style_compiler::style_parser::ParserInput::new("rgb(1,,2)");
    let mut parser = style_compiler::style_parser::Parser::new(&mut input);
    assert!(CssColor::parse(&mut parser).is_err());
}

fn public_fixture() -> Result<(tempfile::TempDir, Config)> {
    let dir = tempfile::tempdir()?;
    let web = dir.path();
    fs::create_dir(web.join("src"))?;
    fs::create_dir(web.join("pkg"))?;
    fs::create_dir_all(web.join("public/legacy"))?;
    fs::write(
        web.join("index.html"),
        "<html><head></head><body><script type=\"module\" src=\"/src/main.js\"></script></body></html>",
    )?;
    fs::write(
        web.join("src/main.js"),
        "import './public.css';import init from '../pkg/app.js';await init();",
    )?;
    fs::write(
        web.join("src/public.css"),
        include_str!("../fixtures/public-urls.css"),
    )?;
    fs::write(
        web.join("pkg/app.js"),
        "export default function init(){return new URL('./app_bg.wasm',import.meta.url)}",
    )?;
    fs::write(web.join("pkg/app_bg.wasm"), b"\0asm\x01\0\0\0")?;
    for name in [
        "bannerWave1.png",
        "bannerWave2.png",
        "backgroundPicture.jpg",
    ] {
        fs::write(web.join("public/legacy").join(name), name)?;
    }
    let mut config = Config::default();
    config.web.entry = "src/main.js".into();
    config.web.licenses.clear();
    Ok((dir, config))
}

#[test]
fn configured_copy_lookup_preserves_missing_file_errors() -> Result<()> {
    let (directory, mut config) = public_fixture()?;
    config
        .web
        .copies
        .insert("assets/copy.txt".into(), "src/missing.txt".into());
    let source = directory.path().join("pkg/app_bg.wasm");
    let mut linker = Linker::new(directory.path(), &directory.path().join("pkg"), config);
    let error = linker.asset(&source).unwrap_err();
    let AssetError::Canonicalize { source, path } = error else {
        panic!("expected the missing copy's canonicalization error");
    };
    assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
    assert_eq!(path, directory.path().join("src/missing.txt"));
    Ok(())
}

#[test]
fn css_diagnostic_retains_its_typed_token_after_input_is_dropped() -> Result<()> {
    use style_compiler::style_optimizer::{
        error::{Error as CssError, ParserError, SelectorError},
        properties::custom::Token,
    };
    let error = {
        let directory = tempfile::tempdir()?;
        fs::write(directory.path().join("main.js"), "import './broken.css';")?;
        fs::write(
            directory.path().join("broken.css"),
            ".box[foo=#broken] { color: red; }",
        )?;
        Linker::new(directory.path(), directory.path(), Config::default())
            .module(&directory.path().join("main.js"))
            .unwrap_err()
    };
    assert!(error.source().is_some());
    let AssetError::CssParse { source, .. } = error else {
        panic!("expected a CSS parse failure");
    };
    let source: &CssError<ParserError<'static>> = &source;
    assert!(source.loc.is_some());
    let ParserError::SelectorError(SelectorError::BadValueInAttr(Token::IDHash(value))) =
        &source.kind
    else {
        panic!("expected the invalid attribute's hash token: {source:?}");
    };
    assert_eq!(value.as_ref(), "broken");
    Ok(())
}

#[test]
fn output_write_failure_reports_its_operation_path_and_source() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("blocked.css");
    fs::create_dir(&path)?;
    let error = write_outputs(
        directory.path(),
        &BTreeMap::from([("blocked.css".into(), b"body{}".to_vec())]),
    )
    .unwrap_err();
    let AssetError::WriteOutput {
        path: failed,
        source,
    } = error
    else {
        panic!("expected an output write failure");
    };
    assert_eq!(failed, path);
    assert_eq!(source.kind(), io::ErrorKind::IsADirectory);
    Ok(())
}

#[test]
fn font_preload_discovered_after_copying_keeps_the_configured_output_name() -> Result<()> {
    let (directory, mut config) = public_fixture()?;
    let web = directory.path();
    fs::create_dir_all(web.join("styles/web-fonts"))?;
    fs::write(web.join("styles/web-fonts/alone.woff2"), b"font")?;
    fs::write(
        web.join("index.html"),
        "<html><head><link rel='preload' href='/styles/web-fonts/alone.woff2'></head><body><script type='module' src='/src/main.js'></script></body></html>",
    )?;
    config.web.copies.insert(
        "assets/fixed.woff2".into(),
        "styles/web-fonts/alone.woff2".into(),
    );
    let (outputs, _) = Linker::new(web, &web.join("pkg"), config).finish(b"{}")?;
    assert_eq!(outputs["assets/fixed.woff2"], b"font");
    assert!(String::from_utf8_lossy(&outputs["index.html"]).contains("href='/assets/fixed.woff2'"));
    assert_eq!(
        outputs
            .keys()
            .filter(|name| name.ends_with(".woff2"))
            .count(),
        1
    );
    Ok(())
}

fn public_manifest(web: &Path, config: &mut Config) -> Result<()> {
    config.web.public_assets_manifest = "public-assets.json".into();
    fs::write(
        web.join(&config.web.public_assets_manifest),
        serde_json::to_vec(&json!({
            "format": 1,
            "files": inventory::files(&web.join("public"))?,
        }))?,
    )?;
    Ok(())
}

fn svg_bytes(length: usize) -> Vec<u8> {
    let prefix = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><!--";
    let suffix = b"--></svg>";
    assert!(length >= prefix.len() + suffix.len());
    let mut bytes = vec![b'x'; length];
    bytes[..prefix.len()].copy_from_slice(prefix);
    bytes[length - suffix.len()..].copy_from_slice(suffix);
    bytes
}

#[test]
fn public_manifest_requires_exact_media_and_nonmedia_source_bytes() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    fs::write(
        web.join("public/legacy/README.md"),
        "Reviewed media sources",
    )?;
    public_manifest(web, &mut config)?;
    Linker::new(web, &web.join("pkg"), config.clone()).finish(b"{}")?;
    let original = fs::read(web.join("public/legacy/bannerWave1.png"))?;
    fs::write(web.join("public/legacy/bannerWave1.png"), "changed bytes")?;
    assert!(
        Linker::new(web, &web.join("pkg"), config.clone())
            .finish(b"{}")
            .unwrap_err()
            .to_string()
            .contains("public assets differ from source inventory")
    );
    fs::write(web.join("public/legacy/bannerWave1.png"), original)?;
    fs::write(web.join("public/added.svg"), "<svg/>")?;
    assert!(
        Linker::new(web, &web.join("pkg"), config.clone())
            .finish(b"{}")
            .is_err()
    );
    fs::remove_file(web.join("public/added.svg"))?;
    fs::remove_file(web.join("public/legacy/README.md"))?;
    assert!(
        Linker::new(web, &web.join("pkg"), config.clone())
            .finish(b"{}")
            .is_err()
    );
    fs::write(
        web.join("public/legacy/README.md"),
        "Reviewed media sources",
    )?;
    let mut files = inventory::files(&web.join("public"))?;
    files.remove("legacy/README.md");
    fs::write(
        web.join("public-assets.json"),
        serde_json::to_vec(&json!({"format":1,"files":files}))?,
    )?;
    assert!(
        Linker::new(web, &web.join("pkg"), config)
            .finish(b"{}")
            .is_err()
    );
    Ok(())
}

#[test]
fn registered_public_media_have_independent_budgets_and_exact_receipts() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    let image = svg_bytes(512 * 1024 + 1);
    fs::write(web.join("public/hero.svg"), &image)?;
    fs::write(
        web.join("public/clip.mp4"),
        b"\0\0\0\x18ftypisom\0\0\0\0isommp42",
    )?;
    fs::write(web.join("public/sound.mp3"), b"ID3\x04\0\0\0\0\0\0")?;
    public_manifest(web, &mut config)?;
    config.web.max_total_bytes = 4096;
    let mut linker = Linker::new(web, &web.join("pkg"), config.clone());
    linker.copy_public()?;
    assert!(linker.public_media.contains("clip.mp4"));
    assert!(linker.public_media.contains("sound.mp3"));
    assert!(linker.public_media.contains("legacy/bannerWave1.png"));
    assert!(linker.public_media.contains("legacy/backgroundPicture.jpg"));
    let (outputs, receipt) = Linker::new(web, &web.join("pkg"), config).finish(b"{}")?;
    assert_eq!(outputs["hero.svg"], image);
    assert_eq!(outputs.values().filter(|bytes| *bytes == &image).count(), 1);
    let delivered = receipt["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["path"] == "hero.svg")
        .collect::<Vec<_>>();
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0]["sha256"], inventory::sha(&image));
    Ok(())
}

#[test]
fn registered_public_media_enforce_four_mib_files_and_eight_mib_total() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    fs::write(
        web.join("public/hero.svg"),
        svg_bytes(MAX_PUBLIC_MEDIA_BYTES + 1),
    )?;
    public_manifest(web, &mut config)?;
    let mut linker = Linker::new(web, &web.join("pkg"), config.clone());
    assert!(
        linker
            .copy_public()
            .unwrap_err()
            .to_string()
            .contains("public media hero.svg exceeds 4194304-byte budget")
    );
    fs::write(
        web.join("public/hero.svg"),
        svg_bytes(MAX_PUBLIC_MEDIA_BYTES),
    )?;
    fs::write(
        web.join("public/second.svg"),
        svg_bytes(MAX_PUBLIC_MEDIA_BYTES),
    )?;
    public_manifest(web, &mut config)?;
    let mut linker = Linker::new(web, &web.join("pkg"), config);
    assert!(
        linker
            .copy_public()
            .unwrap_err()
            .to_string()
            .contains("public media exceed 8388608-byte total budget")
    );
    Ok(())
}

#[test]
fn registered_javascript_and_unregistered_images_keep_original_asset_limits() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    fs::write(web.join("public/app.js"), vec![b' '; 512 * 1024 + 1])?;
    public_manifest(web, &mut config)?;
    let error = Linker::new(web, &web.join("pkg"), config.clone())
        .finish(b"{}")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("asset app.js exceeds 524288-byte budget")
    );
    fs::remove_file(web.join("public/app.js"))?;
    fs::write(web.join("public/hero.svg"), svg_bytes(512 * 1024 + 1))?;
    config.web.public_assets_manifest.clear();
    let error = Linker::new(web, &web.join("pkg"), config.clone())
        .finish(b"{}")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("asset hero.svg exceeds 524288-byte budget")
    );
    Ok(())
}

#[test]
fn media_inventory_does_not_relax_bundle_total_or_reclassify_text_and_fonts() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    fs::write(web.join("public/hero.svg"), svg_bytes(512 * 1024 + 1))?;
    for name in ["README.md", "app.js", "app.css", "font.woff2"] {
        fs::write(web.join("public").join(name), "bundle")?;
    }
    public_manifest(web, &mut config)?;
    config.web.max_total_bytes = 1;
    let mut classification = config.clone();
    classification.web.max_total_bytes = 16 * 1024 * 1024;
    let mut linker = Linker::new(web, &web.join("pkg"), classification);
    linker.copy_public()?;
    for name in ["README.md", "app.js", "app.css", "font.woff2"] {
        assert!(!linker.public_media.contains(name));
    }
    let error = Linker::new(web, &web.join("pkg"), config.clone())
        .finish(b"{}")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("web bundle exceeds 1-byte total budget")
    );
    config.web.max_total_bytes = 16 * 1024 * 1024;
    fs::write(web.join("public/font.woff2"), vec![0; 16 * 1024 * 1024])?;
    public_manifest(web, &mut config)?;
    let error = Linker::new(web, &web.join("pkg"), config)
        .finish(b"{}")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("web bundle exceeds 16777216-byte total budget")
    );
    Ok(())
}

#[test]
fn public_manifest_schema_and_location_cannot_hide_ambiguous_or_untracked_inputs() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    config.web.public_assets_manifest = "public-assets.json".into();
    for invalid in [
        r#"{"format":2,"files":{}}"#,
        r#"{"format":1,"files":{},"ignored":true}"#,
        r#"{"format":1,"files":{"a":"x","a":"y"}}"#,
        r#"{"format":1,"files":{"../outside.svg":"0000000000000000000000000000000000000000000000000000000000000000"}}"#,
    ] {
        fs::write(web.join("public-assets.json"), invalid)?;
        assert!(
            Linker::new(web, &web.join("pkg"), config.clone())
                .finish(b"{}")
                .is_err(),
            "accepted {invalid}"
        );
    }
    for invalid in [
        "../public-assets.json",
        "src/public-assets.json",
        "/tmp/public-assets.json",
    ] {
        config.web.public_assets_manifest = invalid.into();
        assert!(
            Linker::new(web, &web.join("pkg"), config.clone())
                .finish(b"{}")
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn public_css_urls_keep_fixed_paths_without_duplicate_hashed_assets() -> Result<()> {
    let (dir, config) = public_fixture()?;
    let web = dir.path();
    let provenance = serde_json::to_vec(&inventory::files(&web.join("public"))?)?;
    let (outputs, receipt) = Linker::new(web, &web.join("pkg"), config).finish(&provenance)?;
    let css = outputs
        .iter()
        .find(|(name, _)| name.ends_with(".css"))
        .unwrap()
        .1;
    let css = String::from_utf8_lossy(css);
    for name in [
        "bannerWave1.png",
        "bannerWave2.png",
        "backgroundPicture.jpg",
    ] {
        let output = format!("legacy/{name}");
        assert!(css.contains(&format!("/{output}")));
        assert_eq!(
            outputs[&output],
            fs::read(web.join("public").join(&output))?
        );
        assert_eq!(
            outputs
                .values()
                .filter(|bytes| bytes.as_slice() == name.as_bytes())
                .count(),
            1
        );
        let delivered = receipt["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["path"] == output)
            .collect::<Vec<_>>();
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0]["sha256"], inventory::sha(&outputs[&output]));
    }
    assert_eq!(outputs["assets/blog-assets.json"], provenance);
    Ok(())
}

#[test]
fn absolute_imports_prefer_web_files_and_public_fallback_is_bounded() -> Result<()> {
    let (dir, config) = public_fixture()?;
    let web = dir.path();
    fs::write(web.join("priority.svg"), "<svg>web</svg>")?;
    fs::write(web.join("public/priority.svg"), "<svg>public</svg>")?;
    let parent = web.join("src/main.js");
    let linker = Linker::new(web, &web.join("pkg"), config.clone());
    assert_eq!(
        linker.resolve(&parent, "/priority.svg?url")?,
        web.join("priority.svg")
    );
    assert_eq!(
        linker.resolve_with_public_priority(&parent, "/priority.svg", true)?,
        web.join("public/priority.svg")
    );
    assert_eq!(
        linker.resolve(&parent, "/legacy/bannerWave1.png?url")?,
        web.join("public/legacy/bannerWave1.png")
    );
    for url in [
        "/../outside.svg",
        "/legacy/../../outside.svg",
        "//example.org/image.svg",
    ] {
        assert!(linker.resolve(&parent, url).is_err(), "accepted {url}");
    }
    fs::write(
        web.join("src/main.js"),
        "import picture from '/legacy/bannerWave1.png?url';import init from '/pkg/app.js';await init();console.log(picture);",
    )?;
    let stage = tempfile::tempdir()?;
    let pkg = stage.path().join("pkg");
    fs::rename(web.join("pkg"), &pkg)?;
    let (outputs, _) = Linker::new(web, &pkg, config).finish(b"{}")?;
    assert!(
        outputs
            .iter()
            .any(|(name, bytes)| name.starts_with("assets/asset-url-")
                && String::from_utf8_lossy(bytes).contains("/legacy/bannerWave1.png"))
    );
    assert!(
        !outputs
            .keys()
            .any(|name| name.starts_with("assets/bannerWave1-"))
    );
    Ok(())
}

#[test]
fn public_assets_remain_budgeted_and_cannot_shadow_provenance() -> Result<()> {
    let (dir, config) = public_fixture()?;
    let web = dir.path();
    fs::write(
        web.join("public/oversized.bin"),
        vec![0; config.web.max_asset_bytes + 1],
    )?;
    let error = Linker::new(web, &web.join("pkg"), config.clone())
        .finish(b"{}")
        .unwrap_err();
    assert!(error.to_string().contains("asset oversized.bin exceeds"));
    fs::remove_file(web.join("public/oversized.bin"))?;
    fs::create_dir(web.join("public/assets"))?;
    fs::write(web.join("public/assets/blog-assets.json"), "{}")?;
    let error = Linker::new(web, &web.join("pkg"), config)
        .finish(b"{}")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("public asset shadows a generated resource: assets/blog-assets.json")
    );
    Ok(())
}

#[test]
fn embedded_public_urls_are_relative_to_the_css_and_module_outputs() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    config.web.kind = OutputKind::Embedded;
    config.web.relative_urls = true;
    config.web.css_output = "dist/platform.css".into();
    config
        .web
        .entries
        .insert("index.js".into(), "src/main.js".into());
    fs::write(
        web.join("src/main.js"),
        "import './public.css';import picture from '/legacy/bannerWave1.png?url';import init from '../pkg/app.js';await init();console.log(picture);",
    )?;
    let (outputs, _) = Linker::new(web, &web.join("pkg"), config).finish(b"{}")?;
    let css = String::from_utf8_lossy(&outputs["dist/platform.css"]);
    assert!(css.contains("../legacy/bannerWave1.png"));
    assert!(
        outputs
            .iter()
            .any(|(name, bytes)| name.starts_with("assets/asset-url-")
                && String::from_utf8_lossy(bytes).contains("../legacy/bannerWave1.png"))
    );
    assert!(
        !outputs
            .keys()
            .any(|name| name.starts_with("assets/bannerWave1-"))
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn public_fallback_rejects_symlinks_instead_of_reading_outside() -> Result<()> {
    use std::os::unix::fs::symlink;
    let outside = tempfile::tempdir()?;
    fs::write(outside.path().join("outside.svg"), "<svg/>")?;
    let (dir, config) = public_fixture()?;
    let web = dir.path();
    symlink(
        outside.path().join("outside.svg"),
        web.join("public/escape.svg"),
    )?;
    let linker = Linker::new(web, &web.join("pkg"), config);
    let error = linker
        .resolve(&web.join("src/main.js"), "/escape.svg")
        .unwrap_err();
    let diagnostic = diagnostic(&error);
    assert!(
        diagnostic.contains("Symlink input/output is forbidden"),
        "{diagnostic}"
    );
    assert!(linker.outputs.is_empty());
    assert!(
        fs::symlink_metadata(web.join("public/escape.svg"))?
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(outside.path().join("outside.svg"))?, b"<svg/>");
    Ok(())
}

#[test]
fn esm_css_and_wasm_urls_are_hashed_with_matching_preload() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let web = dir.path();
    fs::create_dir(web.join("src"))?;
    fs::create_dir(web.join("pkg"))?;
    fs::write(
        web.join("index.html"),
        "<html><head></head><body><script type=\"module\" src=\"/src/main.js\"></script></body></html>",
    )?;
    fs::write(
        web.join("src/main.js"),
        "import './style.css';\nimport init from '../pkg/app.js';\nawait init();",
    )?;
    fs::write(web.join("src/style.css"), "body{background:url('./a.svg')}")?;
    fs::write(web.join("src/a.svg"), "<svg/>")?;
    fs::write(
        web.join("pkg/app.js"),
        "export default function init(){return new URL('app_bg.wasm',import.meta.url)}",
    )?;
    fs::write(web.join("pkg/app_bg.wasm"), b"\0asm\x01\0\0\0")?;
    let mut config = Config::default();
    config.web.entry = "src/main.js".into();
    config.web.licenses.clear();
    let (outputs, receipt) = Linker::new(web, &web.join("pkg"), config).finish(b"{}")?;
    let html = String::from_utf8(outputs["index.html"].clone())?;
    let wasm = outputs.keys().find(|v| v.ends_with(".wasm")).unwrap();
    assert!(html.contains(wasm));
    assert!(outputs.contains_key(&format!("{wasm}.gz")));
    assert_eq!(receipt["modules"].as_array().unwrap().len(), 2);
    let all_js = outputs
        .iter()
        .filter(|(n, _)| n.ends_with(".js"))
        .map(|(_, b)| String::from_utf8_lossy(b))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all_js.contains(wasm));
    assert!(!all_js.contains("../pkg/"));
    assert!(!all_js.contains("./style.css"));
    Ok(())
}
#[test]
fn css_imports_preserve_repeated_cascade_positions() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let web = dir.path();
    fs::write(web.join("red.css"), ".x{color:red}")?;
    fs::write(web.join("blue.css"), ".x{color:blue}")?;
    fs::write(
        web.join("main.css"),
        "@import './red.css';@import './blue.css';@import './red.css';",
    )?;
    fs::write(
        web.join("main.js"),
        "import './main.css';import './main.css';",
    )?;
    let mut linker = Linker::new(web, web, Config::default());
    linker.module(&web.join("main.js"))?;
    // CSS imports repeat at their authored cascade positions; the ESM
    // module itself still evaluates a side-effect stylesheet only once.
    assert_eq!(linker.css, ".x{color:red}.x{color:#00f}.x{color:red}");
    Ok(())
}

#[test]
fn css_import_expansion_is_bounded_and_cycles_fail() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let web = dir.path();
    fs::write(web.join("leaf.css"), ".x{color:red}")?;
    fs::write(
        web.join("main.css"),
        "@import './leaf.css';@import './leaf.css';",
    )?;
    let mut config = Config::default();
    config.web.max_asset_bytes = 20;
    let error = Linker::new(web, web, config)
        .stylesheet(&web.join("main.css"))
        .unwrap_err();
    assert!(error.to_string().contains("expanded CSS exceeds 20-byte"));
    fs::write(web.join("leaf.css"), "@import './main.css';")?;
    let error = Linker::new(web, web, Config::default())
        .stylesheet(&web.join("main.css"))
        .unwrap_err();
    assert!(error.to_string().contains("circular CSS @import"));
    Ok(())
}

#[test]
fn css_cache_stays_bounded_for_long_import_only_chains() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let web = dir.path();
    fs::write(web.join("0.css"), ".x{color:red}")?;
    for index in 1..32 {
        fs::write(
            web.join(format!("{index}.css")),
            format!("@import './{}.css';", index - 1),
        )?;
    }
    let mut config = Config::default();
    config.web.max_total_bytes = 32;
    let mut linker = Linker::new(web, web, config);
    assert_eq!(linker.stylesheet(&web.join("31.css"))?, ".x{color:red}");
    assert!(linker.css_cache_bytes <= 32);
    assert_eq!(
        linker.css_cache.values().map(String::len).sum::<usize>(),
        linker.css_cache_bytes
    );
    Ok(())
}

#[test]
fn asset_url_modules_preserve_export_names_and_lexical_scope() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let web = dir.path();
    fs::write(web.join("icon.svg"), "<svg/>")?;
    fs::write(
        web.join("main.js"),
        r#"import icon from './icon.svg?url';
           export { default } from './icon.svg?url';
           export { default as "icon-url" } from './icon.svg?url';
           const URL = class { constructor() { throw new Error('captured'); } };
           export { icon, URL };
           export function local(URL) { return new URL('./missing.svg', import.meta.url); }"#,
    )?;
    let mut config = Config::default();
    config.web.relative_urls = true;
    let mut linker = Linker::new(web, web, config);
    let output = linker.module(&web.join("main.js"))?;
    let javascript = std::str::from_utf8(&linker.outputs[&output])?;
    let refs = crate::javascript::references(javascript)?;
    assert_eq!(refs.imports.len(), 3);
    assert!(refs.assets.is_empty());
    assert!(
        refs.imports
            .iter()
            .all(|import| import.url == refs.imports[0].url)
    );
    let url_module = format!("assets/{}", refs.imports[0].url.strip_prefix("./").unwrap());
    let helper = std::str::from_utf8(&linker.outputs[&url_module])?;
    let refs = crate::javascript::references(helper)?;
    assert_eq!(refs.assets.len(), 1, "{helper}");
    assert!(refs.assets[0].0.starts_with("./icon-"));
    assert!(helper.starts_with("export default "), "{helper}");
    assert!(javascript.contains("icon-url"), "{javascript}");
    assert_eq!(
        linker
            .outputs
            .keys()
            .filter(|name| name.starts_with("assets/asset-url-"))
            .count(),
        1,
    );
    Ok(())
}

#[test]
fn budgets_cannot_be_relaxed_and_bare_imports_fail() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("rust-web.toml"),
        "[web]\nmax_asset_bytes=1048576",
    )?;
    assert!(Config::read(dir.path()).is_err());
    fs::write(
        dir.path().join("main.js"),
        "import x from 'unreviewed-package';",
    )?;
    let mut linker = Linker::new(dir.path(), dir.path(), Config::default());
    assert!(linker.module(&dir.path().join("main.js")).is_err());
    Ok(())
}

#[test]
fn embedded_entries_keep_templates_fixed_fonts_and_same_origin_svg() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let web = dir.path().join("src/web");
    let pkg = dir.path().join("staged-pkg");
    fs::create_dir_all(web.join("vendor/test"))?;
    fs::create_dir_all(&pkg)?;
    fs::write(
        web.join("index.html"),
        "__ASSETS_PREFIX__ __INDEX_DATA__ __WASM_PRELOAD__",
    )?;
    fs::write(
        web.join("index.js"),
        "import './platform.js';import init from './pkg/app.js';await init();",
    )?;
    fs::write(
        web.join("platform.js"),
        "import './vendor/test/font.css';export const bridge=1;",
    )?;
    fs::write(
        web.join("vendor/test/font.css"),
        "@font-face{src:url('./font.woff2')}a{mask-image:url(\"data:image/svg+xml,%3Csvg/%3E\")}",
    )?;
    fs::write(web.join("vendor/test/font.woff2"), "font")?;
    fs::write(
        pkg.join("app.js"),
        "export default function init(){return new URL('./app_bg.wasm',import.meta.url)}",
    )?;
    fs::write(pkg.join("app_bg.wasm"), b"\0asm\x01\0\0\0")?;
    let mut config = Config::default();
    config.web.kind = OutputKind::Embedded;
    config.web.relative_urls = true;
    config.web.runtime_root = RuntimeRoot::Product;
    config.web.licenses.clear();
    config.web.css_output = "dist/platform.css".into();
    config
        .web
        .entries
        .insert("index.js".into(), "index.js".into());
    config
        .web
        .entries
        .insert("dist/platform.js".into(), "platform.js".into());
    config
        .web
        .copies
        .insert("dist/font.woff2".into(), "vendor/test/font.woff2".into());
    let (outputs, receipt) = Linker::new(&web, &pkg, config).finish(b"{}")?;
    assert!(!outputs.contains_key("index.html"));
    assert!(outputs.contains_key("index.js"));
    assert!(outputs.contains_key("dist/platform.js"));
    assert_eq!(
        outputs
            .keys()
            .filter(|name| name.ends_with(".woff2"))
            .count(),
        1
    );
    let css = String::from_utf8(outputs["dist/platform.css"].clone())?;
    assert!(!css.contains("data:"));
    assert!(css.contains("../assets/blog-icon-"));
    assert!(css.contains("../dist/font.woff2"));
    assert!(
        receipt["modules"]
            .as_array()
            .unwrap()
            .iter()
            .all(|module| module["path"].as_str().unwrap().starts_with("src/web/"))
    );
    assert_eq!(
        fs::read_to_string(web.join("index.html"))?,
        "__ASSETS_PREFIX__ __INDEX_DATA__ __WASM_PRELOAD__"
    );
    Ok(())
}

#[test]
fn configured_licenses_and_copies_stay_in_authored_web_sources() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let web = dir.path();
    for invalid in [
        "/tmp/external-license",
        "../external-license",
        "dist/license.txt",
        "node_modules/license.txt",
    ] {
        fs::write(
            web.join("rust-web.toml"),
            format!("[web]\nlicenses=[{invalid:?}]\n"),
        )?;
        assert!(Config::read(web).is_err(), "{invalid}");
        config.web.licenses = vec![invalid.into()];
        assert!(
            Linker::new(web, &web.join("pkg"), config.clone())
                .finish(b"{}")
                .is_err(),
            "{invalid}"
        );
    }
    fs::create_dir_all(web.join("custom"))?;
    fs::write(web.join("custom/notice.txt"), "Reviewed custom notice")?;
    fs::write(web.join("custom/static.txt"), "Reviewed custom copy")?;
    config.web.licenses = vec!["custom/notice.txt".into()];
    config
        .web
        .copies
        .insert("assets/static.txt".into(), "custom/static.txt".into());
    let (outputs, _) = Linker::new(web, &web.join("pkg"), config).finish(b"{}")?;
    assert_eq!(outputs["assets/notice.txt"], b"Reviewed custom notice");
    assert_eq!(outputs["assets/static.txt"], b"Reviewed custom copy");
    Ok(())
}

#[cfg(unix)]
#[test]
fn configured_license_symlink_cannot_publish_external_bytes() -> Result<()> {
    let (dir, mut config) = public_fixture()?;
    let external = tempfile::NamedTempFile::new()?;
    fs::write(external.path(), "External bytes must remain private")?;
    fs::create_dir(dir.path().join("custom"))?;
    std::os::unix::fs::symlink(external.path(), dir.path().join("custom/license.txt"))?;
    config.web.licenses = vec!["custom/license.txt".into()];
    let error = Linker::new(dir.path(), &dir.path().join("pkg"), config)
        .finish(b"{}")
        .unwrap_err();
    assert!(diagnostic(&error).contains("Symlink input/output is forbidden"));
    Ok(())
}

#[test]
fn generated_web_trees_cannot_be_linker_sources() -> Result<()> {
    let (dir, config) = public_fixture()?;
    let web = dir.path();
    for generated in [
        "dist/previous.js",
        "node_modules/package/entry.js",
        "custom/node_modules/package/entry.js",
    ] {
        let path = web.join(generated);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, "export const value=1;")?;
        fs::write(web.join("src/main.js"), format!("import '../{generated}';"))?;
        let error = Linker::new(web, &web.join("pkg"), config.clone())
            .finish(b"{}")
            .unwrap_err();
        assert!(
            diagnostic(&error).contains("Generated or ignored Web paths"),
            "{generated}: {error:#}"
        );
    }
    Ok(())
}

#[test]
fn output_admission_is_incremental_and_duplicate_outputs_are_not_counted_twice() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut config = Config::default();
    config.web.max_asset_bytes = 5;
    config.web.max_total_bytes = 5;
    let mut linker = Linker::new(directory.path(), directory.path(), config);
    linker.generated_output("a.js".into(), b"aaa".to_vec())?;
    linker.generated_output("a.js".into(), b"aaa".to_vec())?;
    assert!(
        linker
            .generated_output("b.js".into(), b"bbb".to_vec())
            .is_err()
    );
    assert!(!linker.outputs.contains_key("b.js"));
    linker.generated_output("b.js".into(), b"bb".to_vec())?;
    linker.budgets()?;
    assert_eq!(linker.outputs.values().map(Vec::len).sum::<usize>(), 5);
    Ok(())
}

#[test]
fn oversized_public_output_fails_before_reading_later_files() -> Result<()> {
    let (directory, config) = public_fixture()?;
    fs::write(
        directory.path().join("public/a.txt"),
        vec![b'x'; 512 * 1024 + 1],
    )?;
    fs::File::create(directory.path().join("public/z-too-large.txt"))?
        .set_len(inventory::MAX_FILE + 1)?;
    let mut linker = Linker::new(directory.path(), &directory.path().join("pkg"), config);
    let error = linker.copy_public().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("asset a.txt exceeds 524288-byte budget")
    );
    assert!(!linker.outputs.contains_key("a.txt"));
    assert!(linker.outputs.is_empty());
    Ok(())
}
