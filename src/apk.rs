use anyhow::Result;

pub fn get_apktool() -> Result<()> {
    crate::utils::download_file(
        "https://github.com/iBotPeaches/Apktool/releases/download/v2.11.0/apktool_2.11.0.jar",
        "balapatch/apktool.jar",
    )
    .expect("fuck");

    if std::fs::exists("balapatch/apktool.jar")? {
        println!("Apktool downloaded successfully!");
    }

    Ok(())
}