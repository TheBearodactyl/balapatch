//! Programmatic wrapper for lovely :3
//! Lets you patch individual files with specific
//! `lovely.toml` files instead of needing a
//! `Mods` directory.
//! ----------
//! This will help quite a lot with
//! patching the android code since
//! you can't use a `Mods` dir on
//! android without quite a lot
//! of tweaks

use crop::Rope;
use std::fs;
use std::path::{Path, PathBuf};

use lovely_core::patch::{
    copy::CopyPatch, module::ModulePatch, pattern::PatternPatch, regex::RegexPatch, Patch,
    PatchFile,
};

#[derive(Debug)]
pub enum PatchError {
    IoError(std::io::Error),
    ParseError(toml::de::Error),
    Other(String),
}

impl From<std::io::Error> for PatchError {
    fn from(err: std::io::Error) -> Self {
        PatchError::IoError(err)
    }
}

impl From<toml::de::Error> for PatchError {
    fn from(err: toml::de::Error) -> Self {
        PatchError::ParseError(err)
    }
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchError::IoError(e) => write!(f, "IO Error: {}", e),
            PatchError::ParseError(e) => write!(f, "TOML Parse Error: {}", e),
            PatchError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

pub struct Patcher {
    source_file: Option<PathBuf>,
    patch_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    target_name: Option<String>,
    module_handler: Option<Box<dyn ModuleHandler>>,
}

pub trait ModuleHandler {
    fn handle_module_patch(
        &self,
        module_patch: &ModulePatch,
        target: &str,
        patch_dir: &Path,
    ) -> bool;
}

impl Patcher {
    pub fn new() -> Self {
        Patcher {
            source_file: None,
            patch_file: None,
            output_file: None,
            target_name: None,
            module_handler: None,
        }
    }

    pub fn source<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.source_file = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn patch<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.patch_file = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn output<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_file = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.target_name = Some(name.as_ref().to_string());
        self
    }

    pub fn module_handler<H: ModuleHandler + 'static>(mut self, handler: H) -> Self {
        self.module_handler = Some(Box::new(handler));
        self
    }

    pub fn patch_file(&self) -> Result<(), PatchError> {
        let source_path = self
            .source_file
            .as_ref()
            .ok_or_else(|| PatchError::Other("Source file not specified".to_string()))?;
        let patch_path = self
            .patch_file
            .as_ref()
            .ok_or_else(|| PatchError::Other("Patch file not specified".to_string()))?;
        let output_path = self
            .output_file
            .as_ref()
            .ok_or_else(|| PatchError::Other("Output file not specified".to_string()))?;

        let target = match &self.target_name {
            Some(name) => name.clone(),
            None => source_path
                .file_name()
                .ok_or_else(|| {
                    PatchError::Other("Could not determine source filename".to_string())
                })?
                .to_string_lossy()
                .to_string(),
        };

        let source_content = fs::read_to_string(source_path)?;
        let mut rope = Rope::from(source_content);
        let patch_content = fs::read_to_string(patch_path)?;
        let patch_file: PatchFile = toml::from_str(&patch_content)?;
        let patch_dir = patch_path.parent().unwrap_or(Path::new("."));

        self.apply_patches(&target, &mut rope, &patch_file, patch_dir)?;

        fs::write(output_path, rope.to_string())?;

        Ok(())
    }

    fn apply_patches(
        &self,
        target: &str,
        rope: &mut Rope,
        patch_file: &PatchFile,
        patch_dir: &Path,
    ) -> Result<(), PatchError> {
        // Process variable interpolation for the content in all patches
        let vars = &patch_file.vars;
        let mut applied_count = 0;

        for patch in &patch_file.patches {
            let result = match patch {
                Patch::Copy(copy_patch) => {
                    self.apply_copy_patch(target, rope, copy_patch, patch_dir)?;
                    true
                }
                Patch::Pattern(pattern_patch) => {
                    self.apply_pattern_patch(target, rope, pattern_patch, patch_dir)?;
                    true
                }
                Patch::Regex(regex_patch) => {
                    self.apply_regex_patch(target, rope, regex_patch, patch_dir)?;
                    true
                }
                Patch::Module(module_patch) => {
                    self.apply_module_patch(target, module_patch, patch_dir)?
                }
            };

            if result {
                applied_count += 1;
            }
        }

        if !vars.is_empty() {
            let mut content = rope.to_string();
            let mut lines = content
                .split('\n')
                .map(String::from)
                .collect::<Vec<String>>();

            for line in &mut lines {
                lovely_core::patch::vars::apply_var_interp(line, vars);
            }

            let new_content = lines.join("\n");
            *rope = Rope::from(new_content);
        }

        println!("Applied {} patches to '{}'", applied_count, target);
        Ok(())
    }

    fn apply_copy_patch(
        &self,
        target: &str,
        rope: &mut Rope,
        patch: &CopyPatch,
        patch_dir: &Path,
    ) -> Result<(), PatchError> {
        if target != patch.target {
            return Ok(());
        }

        let sources = patch
            .sources
            .iter()
            .map(|source| {
                if source.is_absolute() {
                    source.clone()
                } else {
                    patch_dir.join(source)
                }
            })
            .collect::<Vec<PathBuf>>();

        let temp_patch = CopyPatch {
            position: match patch.position {
                lovely_core::patch::copy::CopyPosition::Prepend => {
                    lovely_core::patch::copy::CopyPosition::Prepend
                }
                lovely_core::patch::copy::CopyPosition::Append => {
                    lovely_core::patch::copy::CopyPosition::Append
                }
            },
            target: patch.target.clone(),
            sources,
        };

        temp_patch.apply(target, rope, patch_dir);

        Ok(())
    }

    fn apply_pattern_patch(
        &self,
        target: &str,
        rope: &mut Rope,
        patch: &PatternPatch,
        patch_dir: &Path,
    ) -> Result<(), PatchError> {
        if target != patch.target {
            return Ok(());
        }

        patch.apply(target, rope, patch_dir);
        Ok(())
    }

    fn apply_regex_patch(
        &self,
        target: &str,
        rope: &mut Rope,
        patch: &RegexPatch,
        patch_dir: &Path,
    ) -> Result<(), PatchError> {
        if target != patch.target {
            return Ok(());
        }

        patch.apply(target, rope, patch_dir);
        Ok(())
    }

    fn apply_module_patch(
        &self,
        target: &str,
        patch: &ModulePatch,
        patch_dir: &Path,
    ) -> Result<bool, PatchError> {
        if target != patch.before {
            return Ok(false);
        }

        if let Some(handler) = &self.module_handler {
            Ok(handler.handle_module_patch(patch, target, patch_dir))
        } else {
            let module_path = if patch.source.is_absolute() {
                patch.source.clone()
            } else {
                patch_dir.join(&patch.source)
            };

            match fs::read_to_string(&module_path) {
                Ok(content) => {
                    println!(
                        "Module patch found for '{}'. Module content from {} is available but no handler is registered.",
                        target,
                        module_path.display()
                    );
                    Ok(false)
                }
                Err(e) => {
                    println!(
                        "Warning: Could not read module file at {}: {}",
                        module_path.display(),
                        e
                    );
                    Ok(false)
                }
            }
        }
    }
}

impl Default for Patcher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoggingModuleHandler;

impl ModuleHandler for LoggingModuleHandler {
    fn handle_module_patch(
        &self,
        module_patch: &ModulePatch,
        target: &str,
        patch_dir: &Path,
    ) -> bool {
        let module_path = if module_patch.source.is_absolute() {
            module_patch.source.clone()
        } else {
            patch_dir.join(&module_patch.source)
        };

        match fs::read_to_string(&module_path) {
            Ok(content) => {
                println!(
                    "Module '{}' from file {} would be injected before '{}' {}",
                    module_patch.name,
                    module_path.display(),
                    target,
                    if module_patch.load_now {
                        "and loaded immediately"
                    } else {
                        ""
                    }
                );
                true
            }
            Err(e) => {
                println!(
                    "Warning: Could not read module file at {}: {}",
                    module_path.display(),
                    e
                );
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_simple_file() -> anyhow::Result<(), PatchError> {
        Patcher::new()
            .patch("D:\\Projects\\woah\\balapatch\\lovely.toml")
            .source("D:\\Projects\\woah\\balapatch\\main.lua")
            .output("D:\\Projects\\woah\\balapatch\\skibidi.lua")
            .module_handler(LoggingModuleHandler)
            .patch_file()
    }

    #[test]
    fn multi_file_patch() {}
}
