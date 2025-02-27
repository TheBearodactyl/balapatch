use anyhow::{anyhow, Result};
use tracing::info;

pub async fn has_apktool() -> Result<()> {
    if which::which("apktool").is_err() {
        return Err(anyhow!("failed to find an apktool install..."));
    }
    
    Ok(())
}

pub async fn get_apktool() -> Result<()> {
    if has_apktool().await.is_err() {
        crate::balapatch::utils::misc::download_file(
            "https://github.com/iBotPeaches/Apktool/releases/download/v2.11.0/apktool_2.11.0.jar",
            "balapatch/apktool.jar",
        )
        .await
        .expect("Failed to download Apktool");

        if std::fs::exists("../../balapatch/apktool.jar")? {
            info!("Apktool downloaded successfully!");
        }
    } else {
        info!("Apktool already installed, using existing copy!");
    }

    Ok(())
}
