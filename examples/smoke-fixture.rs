//! Isolated test provisioning; never included in a release package.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("fixture ROOT required"))?,
    );
    match std::env::args().nth(2).as_deref() {
        None => {
            xocs::offline::initialize(
                &root,
                &root.join("config/config.json"),
                "admin",
                "TemporaryPassphrase123",
            )
            .await?;
        }
        Some("check") => {}
        Some(_) => anyhow::bail!("unknown fixture action"),
    }
    xocs::offline::check(&root, &root.join("config/config.json")).await?;
    Ok(())
}
