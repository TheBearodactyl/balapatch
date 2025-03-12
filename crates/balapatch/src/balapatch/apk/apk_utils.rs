#![allow(unused, clippy::manual_ignore_case_cmp)]

use crate::balapatch::utils::grammar_police::GrammarPolice;
use anyhow::anyhow;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ApkUtils {
    apk_path: PathBuf,
}

impl ApkUtils {
    pub fn new(apk_path: PathBuf) -> Result<Self, anyhow::Error> {
        if !apk_path.exists() {
            return Err(anyhow!("Path not found or insufficient permissions"));
        }

        if !apk_path.is_file() {
            return if apk_path.is_dir() {
                Err(anyhow!("Expected a file, found a directory."))
            } else {
                Err(anyhow!("Expected a file, found a symlink."))
            };
        }

        if apk_path.extension().unwrap() != "apk" {
            let formatted_extension = GrammarPolice::new(
                str::to_string(match apk_path.extension().unwrap().to_str() {
                    Some(x) => x,
                    None => todo!(),
                })
                .as_str(),
            )
            .into_inner();

            return Err(anyhow!(
                "Expected an APK file, found {}",
                formatted_extension
            ));
        }

        Ok(Self { apk_path })
    }

    pub fn new_unchecked(apk_path: PathBuf) -> Self {
        Self { apk_path }
    }

    pub fn extract_to_dir(&self, out: PathBuf) -> anyhow::Result<()> {
        if !out.exists() {
            std::fs::create_dir_all(out.as_path())?;
        }

        if !self.apk_path.exists() {
            return Err(anyhow!("Path not found or insufficient permissions"));
        }

        let zip_contents = Cursor::new(std::fs::read(out.as_path())?);

        zip_extract::extract(zip_contents, out.as_path(), false);

        Ok(())
    }
}
