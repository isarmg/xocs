use super::*;
#[test]
fn runtime_receipt_uses_target_normal_tree_without_host_build_or_dev_packages() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let frontend = temp.path().join("src/browser");
    std::fs::create_dir_all(frontend.join("src"))?;
    std::fs::write(frontend.join("src/lib.rs"), "")?;
    std::fs::write(
        frontend.join("Cargo.toml"),
        "[package]\nname='browser'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nruntime={path='../../src/foundation/runtime-1.0.0'}\nhost_macro={path='host_macro'}\n[build-dependencies]\nbuild_only={path='build_only'}\n[dev-dependencies]\ndev_only={path='dev_only'}\n",
    )?;
    for name in ["runtime", "host_macro", "build_only", "dev_only"] {
        let path = if name == "runtime" {
            temp.path().join("src/foundation/runtime-1.0.0")
        } else {
            frontend.join(name)
        };
        std::fs::create_dir_all(path.join("src"))?;
        std::fs::write(path.join("src/lib.rs"), "")?;
        std::fs::write(
            path.join("Cargo.toml"),
            format!(
                "[package]\nname='{name}'\nversion='1.0.0'\nedition='2024'\n{}",
                if name == "host_macro" {
                    "[lib]\nproc-macro=true\n"
                } else {
                    ""
                }
            ),
        )?;
    }
    let lock = "version = 4\n[[package]]\nname='browser'\nversion='1.0.0'\ndependencies=['runtime','host_macro','build_only','dev_only']\n[[package]]\nname='runtime'\nversion='1.0.0'\n[[package]]\nname='host_macro'\nversion='1.0.0'\n[[package]]\nname='build_only'\nversion='1.0.0'\n[[package]]\nname='dev_only'\nversion='1.0.0'\n";
    std::fs::write(temp.path().join("Cargo.lock"), lock)?;
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers=['src/browser']\nresolver='3'\n",
    )?;
    let options = Options {
        root: temp.path().to_owned(),
        offline: true,
        debug: false,
        dist: None,
    };
    let result = runtime(&options, &json!({"browser_inputs":{},"patched_inputs":{}}))?;
    let names = result["graph"]["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, ["browser", "runtime"]);
    assert_eq!(result["lockfile"], "Cargo.lock");
    assert_eq!(result["profile"], "web-release");
    assert!(
        result["graph"]["packages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|package| package["manifest"] == "src/foundation/runtime-1.0.0/Cargo.toml")
    );
    assert!(!serde_json::to_string(&result)?.contains(temp.path().to_str().unwrap()));
    Ok(())
}
fn fixture() -> Result<(tempfile::TempDir, Value)> {
    let temp = tempfile::tempdir()?;
    let frontend = temp.path().join("src/browser");
    std::fs::create_dir_all(&frontend)?;
    std::fs::write(frontend.join("Cargo.toml"), "test")?;
    let graph = temp
        .path()
        .join("src/foundation/interface/reactivity/Cargo.toml");
    std::fs::create_dir_all(graph.parent().unwrap())?;
    std::fs::write(&graph, "test")?;
    for name in ["dom", "macros"] {
        std::fs::create_dir_all(temp.path().join("src/interface").join(name))?;
        std::fs::write(
            temp.path()
                .join("src/interface")
                .join(name)
                .join("Cargo.toml"),
            "test",
        )?;
    }
    let facade = temp.path().join("src/interface/Cargo.toml");
    std::fs::write(&facade, "test")?;
    let core = temp.path().join("src/interface/dom/Cargo.toml");
    let macros = temp.path().join("src/interface/macros/Cargo.toml");
    let value = json!({"packages":[
        {"id":"root","name":"product-web","version":"1","source":null,"manifest_path":frontend.join("Cargo.toml"),"targets":[{"kind":["cdylib"],"src_path":frontend.join("Cargo.toml")}]},
        {"id":"core","name":"blog-dom","version":"0.8.19","source":null,"manifest_path":core,"targets":[{"kind":["lib"],"src_path":core}]},
        {"id":"macro","name":"blog-component-macros","version":"0.8.17","source":null,"manifest_path":macros,"targets":[{"kind":["proc-macro"],"src_path":macros}]},
        {"id":"facade","name":"blog-interface","version":"1.0.0","source":null,"manifest_path":facade,"targets":[{"kind":["lib"],"src_path":facade}]},
        {"id":"graph","name":"blog-interface-reactivity","version":"0.2","source":null,"manifest_path":graph,"targets":[{"kind":["lib"],"src_path":graph}]}
    ],"resolve":{"root":null,"nodes":[
        {"id":"root","dependencies":["facade"],"features":["default"]},
        {"id":"core","dependencies":["macro","graph"],"features":["csr"]},
        {"id":"macro","dependencies":[],"features":["csr"]},
        {"id":"facade","dependencies":["core"],"features":[]},
        {"id":"graph","dependencies":[],"features":["default","effects"]}
    ]}});
    Ok((temp, value))
}
#[test]
fn csr_graph_distinguishes_host_macros_and_unrelated_default_features() -> Result<()> {
    let (temp, value) = fixture()?;
    let receipt = validate(&value, &temp.path().join("src/browser"))?;
    assert_eq!(receipt["integrated"][1]["role"], "host-proc-macro");
    Ok(())
}
#[test]
fn csr_graph_accepts_only_an_exact_csr_default_alias() -> Result<()> {
    let (temp, mut value) = fixture()?;
    for (index, name) in [(1, "dom"), (2, "macros")] {
        std::fs::write(
            temp.path()
                .join("src/interface")
                .join(name)
                .join("Cargo.toml"),
            "[features]\ndefault=['csr']\ncsr=[]\n",
        )?;
        value["resolve"]["nodes"][index]["features"] = json!(["csr", "default"]);
    }
    validate(&value, &temp.path().join("src/browser"))?;
    std::fs::write(
        temp.path().join("src/interface/dom/Cargo.toml"),
        "[features]\ndefault=['csr','ssr']\ncsr=[]\nssr=[]\n",
    )?;
    assert!(validate(&value, &temp.path().join("src/browser")).is_err());
    Ok(())
}
#[test]
fn csr_graph_rejects_upstream_framework_beside_integrated_runtime() -> Result<()> {
    let (temp, mut value) = fixture()?;
    value["packages"].as_array_mut().unwrap().push(json!({
        "id":"upstream","name":"leptos","version":"0.8.19","source":"registry",
        "manifest_path":"registry","targets":[{"kind":["lib"]}]
    }));
    value["resolve"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "id":"upstream","dependencies":[],"features":["csr"]
        }));
    value["resolve"]["nodes"][0]["dependencies"]
        .as_array_mut()
        .unwrap()
        .push(json!("upstream"));
    assert!(validate(&value, &temp.path().join("src/browser")).is_err());
    Ok(())
}
#[test]
fn csr_graph_rejects_full_and_hydration_and_registry_runtime() -> Result<()> {
    let (temp, value) = fixture()?;
    let mut full = value.clone();
    full["resolve"]["nodes"][1]["features"] = json!(["csr", "full"]);
    assert!(validate(&full, &temp.path().join("src/browser")).is_err());
    let mut hydration = value.clone();
    hydration["resolve"]["nodes"][4]["features"] = json!(["default", "hydration"]);
    assert!(validate(&hydration, &temp.path().join("src/browser")).is_err());
    let mut registry = value.clone();
    registry["packages"][1]["source"] = json!("registry");
    assert!(validate(&registry, &temp.path().join("src/browser")).is_err());
    let mut registry_facade = value;
    registry_facade["packages"][3]["source"] = json!("registry");
    assert!(validate(&registry_facade, &temp.path().join("src/browser")).is_err());
    Ok(())
}
#[test]
fn csr_graph_selects_browser_manifest_independently_of_workspace_root() -> Result<()> {
    let (temp, mut value) = fixture()?;
    value["resolve"]["root"] = json!("native-root");
    value["packages"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"native-root","name":"leptos","version":"1","source":"registry"}));
    value["resolve"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"native-root","dependencies":[],"features":["ssr"]}));
    validate(&value, &temp.path().join("src/browser"))?;
    Ok(())
}

#[test]
fn csr_graph_rejects_external_dependency_target_even_with_local_manifest() -> Result<()> {
    let (temp, mut value) = fixture()?;
    let external = tempfile::NamedTempFile::new()?;
    value["packages"][4]["targets"][0]["src_path"] = json!(external.path());
    assert!(validate(&value, &temp.path().join("src/browser")).is_err());
    Ok(())
}
