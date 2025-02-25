use anyhow::Result;
use tracing::info;

pub async fn get_apktool() -> Result<()> {
    if !std::fs::exists("../../balapatch/apktool.jar")? {
        crate::balapatch::utils::download_file(
            "https://github.com/iBotPeaches/Apktool/releases/download/v2.11.0/apktool_2.11.0.jar",
            "balapatch/apktool.jar",
        )
        .await
        .expect("Failed to download Apktool");

        if std::fs::exists("../../balapatch/apktool.jar")? {
            info!("Apktool downloaded successfully!");
        }
    } else {
        info!("Apktool already downloaded, using existing copy!");
    }

    Ok(())
}
