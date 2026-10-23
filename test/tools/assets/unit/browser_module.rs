use super::*;
use anyhow::Context;

#[test]
fn emits_browser_initialization_and_default_url_without_typescript() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let input = temporary.path().join("empty.wasm");
    let output = temporary.path().join("output");
    std::fs::write(&input, b"\0asm\x01\0\0\0")?;
    generate(&input, &output, "browser")?;
    let javascript = std::fs::read_to_string(output.join("browser.js"))?;
    assert!(javascript.contains("export default") || javascript.contains("__wbg_init as default"));
    assert!(javascript.contains("new URL"));
    assert!(!javascript.contains("import * as wasm from"));
    assert!(!output.join("browser.d.ts").exists());
    Ok(())
}

#[test]
fn invalid_module_names_do_not_create_outputs() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let output = temporary.path().join("output");
    for name in ["", "../outside", "nested/name", "name.js"] {
        assert!(generate(&temporary.path().join("unused.wasm"), &output, name).is_err());
        assert!(!output.exists());
    }
    Ok(())
}

#[test]
fn rejects_incompatible_wire_schema_before_decoding_programs() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let input = temporary.path().join("stale.wasm");
    let output = temporary.path().join("output");
    let name = b"__wasm_bindgen_unstable";
    let header = br#"{"schema_version":"old-browser-abi","version":"diagnostic-only"}"#;
    let mut section = vec![u8::try_from(name.len())?];
    section.extend_from_slice(name);
    section.extend_from_slice(&u32::try_from(header.len())?.to_le_bytes());
    section.extend_from_slice(header);
    // Both lengths fit in one unsigned LEB128 byte. No encoded program is
    // present: schema validation must fail before attempting to decode one.
    ensure!(section.len() < 128);
    let mut module = b"\0asm\x01\0\0\0".to_vec();
    module.extend_from_slice(&[0, u8::try_from(section.len())?]);
    module.extend_from_slice(&section);
    std::fs::write(&input, module)?;
    let error = generate(&input, &output, "browser").unwrap_err();
    let message = format!("{error:#}");
    assert!(
        message.contains("Browser binding schema mismatch"),
        "{message}"
    );
    assert!(message.contains("old-browser-abi"), "{message}");
    assert!(message.contains(crate::BRIDGE_ABI), "{message}");
    assert!(!output.join("browser.js").exists());
    Ok(())
}

#[test]
fn generated_initializers_execute_current_object_contract() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    // (module (global (export "starts") (mut i32) (i32.const 0))
    //   (func $start global.get 0 i32.const 1 i32.add global.set 0)
    //   (start $start))
    let module = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x02\x01\0\x06\x06\x01\x7f\x01\x41\0\x0b\x07\x0a\x01\x06starts\x03\0\x08\x01\0\x0a\x0b\x01\x09\0\x23\0\x41\x01\x6a\x24\0\x0b";
    for (name, features) in [
        ("core", &b""[..]),
        (
            "multivalue",
            &b"\0\x1d\x0ftarget_features\x01+\x0amultivalue"[..],
        ),
    ] {
        let input = temporary.path().join(format!("{name}.wasm"));
        let output = temporary.path().join(name);
        std::fs::write(&input, [module.as_slice(), features].concat())?;
        generate(&input, &output, "browser")?;
        std::fs::write(output.join("package.json"), r#"{"type":"module"}"#)?;
        let script = output.join("initialization.mjs");
        std::fs::write(&script, include_str!("../browser-initialization.mjs"))?;
        let result = std::process::Command::new("node")
            .arg(&script)
            .output()
            .context("Node.js 26.7+ is required for generated browser initialization tests")?;
        ensure!(
            result.status.success(),
            "Generated {name} browser initialization failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
        );
    }
    Ok(())
}
