<<<<<<< HEAD
use anyhow::Result;
=======
use anyhow::{anyhow, Result};
>>>>>>> 2c49caf265594eab24344216ff042097dc9d287e
use std::path::PathBuf;
use tracing::info;

pub async fn has_apktool() -> Result<PathBuf, ()> {
    if let Ok(apktool_path) = which::which("apktool") {
        Ok(apktool_path)
    } else {
        Err(())
    }
}

<<<<<<< HEAD
pub async fn get_apktool() -> Result<(), String> {
=======
pub async fn get_apktool() -> anyhow::Result<(), String> {
>>>>>>> 2c49caf265594eab24344216ff042097dc9d287e
    if has_apktool().await.is_err() {
        crate::balapatch::utils::misc::download_file(
            "https://github.com/iBotPeaches/Apktool/releases/download/v2.11.0/apktool_2.11.0.jar",
            "balapatch/apktool.jar",
        )
        .await
        .expect("Failed to download Apktool");

        if std::fs::exists("../../balapatch/apktool.jar").is_ok() {
            info!("Apktool downloaded successfully!");
        }
    } else {
        info!("Apktool already installed, using existing copy!");
    }

    Ok(())
}
