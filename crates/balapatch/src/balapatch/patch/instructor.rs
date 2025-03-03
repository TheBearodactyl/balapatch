//! lmao this is mostly gonna go unused
//! still fun to write

use anyhow::{Context, Error, Result};
use smali::find_smali_files;
use smali::types::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A trait for filtering smali classes
pub trait ClassFilter {
    fn matches(&self, class: &SmaliClass) -> bool;
}

/// Function type for generating instructions
pub type InstructionGenerator = Box<dyn Fn() -> Vec<SmaliInstruction>>;

/// A struct representing a patch to be applied to a smali method
pub struct MethodPatch {
    /// Filter to select methods to patch
    pub method_filter: Box<dyn Fn(&SmaliMethod) -> bool>,
    /// Function that generates new instructions for the method
    pub instruction_generator: InstructionGenerator,
    /// New locals count (if None, keeps original)
    pub locals: Option<u32>,
}

/// A struct representing a patch to be applied to smali classes
pub struct ClassPatch {
    /// Filter to select classes to patch
    pub class_filter: Box<dyn ClassFilter>,
    /// Method patches to apply to matching classes
    pub method_patches: Vec<MethodPatch>,
}

/// The main struct for patching APK files
pub struct Instructor {
    /// Path to the APK file to patch
    apk_path: PathBuf,
    /// Collection of patches to apply
    patches: Vec<ClassPatch>,
    /// Output APK path
    output_path: PathBuf,
    /// Working directory for unpacked APK
    work_dir: PathBuf,
}

impl Instructor {
    /// Create a new Instructor instance for the given APK file
    pub fn new<P: AsRef<Path>>(apk_path: P) -> Self {
        let apk_path = apk_path.as_ref().to_path_buf();
        let file_stem = apk_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Instructor {
            apk_path,
            patches: Vec::new(),
            output_path: PathBuf::from(format!("{}_patched.apk", file_stem)),
            work_dir: PathBuf::from(format!("{}_unpacked", file_stem)),
        }
    }

    /// Set the output path for the patched APK
    pub fn with_output<P: AsRef<Path>>(mut self, output_path: P) -> Self {
        self.output_path = output_path.as_ref().to_path_buf();
        self
    }

    /// Set the working directory for unpacking the APK
    pub fn with_work_dir<P: AsRef<Path>>(mut self, work_dir: P) -> Self {
        self.work_dir = work_dir.as_ref().to_path_buf();
        self
    }

    /// Add a patch to be applied
    pub fn add_patch(mut self, patch: ClassPatch) -> Self {
        self.patches.push(patch);
        self
    }

    /// Apply all patches and create a modified APK
    pub fn apply(self) -> Result<()> {
        // Unpack the APK
        self.unpack_apk()?;

        // Find smali files
        let mut smali_path = self.work_dir.clone();
        smali_path.push("smali");

        let mut classes = find_smali_files(&smali_path).context("Failed to find smali files")?;

        println!("Loaded {} smali classes", classes.len());

        // Track if any modifications were made
        let mut modified = false;

        // Apply patches
        for class in classes.iter_mut() {
            for patch in &self.patches {
                if patch.class_filter.matches(class) {
                    println!("Applying patch to class: {}", class.name.as_java_type());

                    for method_patch in &patch.method_patches {
                        let filter = &method_patch.method_filter;

                        for method in class.methods.iter_mut() {
                            if filter(method) {
                                println!("  Patching method: {}", method.name);
                                // Generate new instructions for this method
                                method.instructions = (method_patch.instruction_generator)();

                                if let Some(locals) = method_patch.locals {
                                    method.locals = locals;
                                }

                                modified = true;
                            }
                        }
                    }
                }
            }

            // Save the class (modified or not)
            class.save().context("Failed to save smali class")?;
        }

        if !modified {
            println!("Warning: No classes were modified by the patches");
        }

        // Repack the APK
        self.repack_apk()?;

        Ok(())
    }

    /// Unpack the APK using apktool
    fn unpack_apk(&self) -> std::result::Result<String, Error> {
        println!("Unpacking APK to {}", self.work_dir.display());

        execute_command(
            "apktool",
            &[
                "decode",
                "-f",
                &self.apk_path.to_string_lossy(),
                "-o",
                &self.work_dir.to_string_lossy(),
            ],
        )
        .context("Failed to unpack APK with apktool")
    }

    /// Repack the APK using apktool
    fn repack_apk(&self) -> std::result::Result<String, Error> {
        println!("Repacking APK to {}", self.output_path.display());

        execute_command(
            "apktool",
            &[
                "build",
                &self.work_dir.to_string_lossy(),
                "-o",
                &self.output_path.to_string_lossy(),
            ],
        )
        .context("Failed to repack APK with apktool")
    }
}

/// Execute a command with the given arguments and return the stdout if successful
fn execute_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute command: {} {:?}", cmd, args))?;

    if output.status.success() {
        let stdout =
            String::from_utf8(output.stdout).context("Failed to parse command output as UTF-8")?;
        Ok(stdout)
    } else {
        let stderr =
            String::from_utf8(output.stderr).context("Failed to parse error output as UTF-8")?;

        anyhow::bail!(
            "Command '{}' failed with status code {:?}.\nArgs: {:?}\nError: {}",
            cmd,
            output.status,
            args,
            stderr
        )
    }
}

/// Convenience struct for filtering classes by name pattern
pub struct ClassNameFilter {
    pattern: String,
}

impl ClassNameFilter {
    /// Create a new filter that matches classes whose name contains the given pattern
    pub fn contains(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }
}

impl ClassFilter for ClassNameFilter {
    fn matches(&self, class: &SmaliClass) -> bool {
        class.name.as_java_type().contains(&self.pattern)
    }
}

/// Convenience struct for filtering classes by method signature patterns
pub struct MethodSignatureFilter {
    method_count: Option<usize>,
    field_count: Option<usize>,
    required_method_signatures: Vec<(TypeSignature, usize)>, // (return_type, arg_count)
}

impl MethodSignatureFilter {
    pub fn new() -> Self {
        Self {
            method_count: None,
            field_count: None,
            required_method_signatures: Vec::new(),
        }
    }

    pub fn with_method_count(mut self, count: usize) -> Self {
        self.method_count = Some(count);
        self
    }

    pub fn with_field_count(mut self, count: usize) -> Self {
        self.field_count = Some(count);
        self
    }

    pub fn require_method_signature(
        mut self,
        return_type: TypeSignature,
        arg_count: usize,
    ) -> Self {
        self.required_method_signatures
            .push((return_type, arg_count));
        self
    }
}

impl ClassFilter for MethodSignatureFilter {
    fn matches(&self, class: &SmaliClass) -> bool {
        // Check method count if specified
        if let Some(count) = self.method_count {
            if class.methods.len() != count {
                return false;
            }
        }

        // Check field count if specified
        if let Some(count) = self.field_count {
            if class.fields.len() != count {
                return false;
            }
        }

        // Check for required method signatures
        for (return_type, arg_count) in &self.required_method_signatures {
            let has_matching_method = class.methods.iter().any(|m| {
                // Based on original code, it seems the return type is just "signature.result"
                m.signature.return_type == *return_type && m.signature.args.len() == *arg_count
            });

            if !has_matching_method {
                return false;
            }
        }

        true
    }
}

/// Complex filter that combines multiple filters with AND logic
pub struct AndFilter {
    filters: Vec<Box<dyn ClassFilter>>,
}

impl AndFilter {
    pub fn new(filters: Vec<Box<dyn ClassFilter>>) -> Self {
        Self { filters }
    }
}

impl ClassFilter for AndFilter {
    fn matches(&self, class: &SmaliClass) -> bool {
        self.filters.iter().all(|f| f.matches(class))
    }
}

/// Complex filter that combines multiple filters with OR logic
pub struct OrFilter {
    filters: Vec<Box<dyn ClassFilter>>,
}

impl OrFilter {
    pub fn new(filters: Vec<Box<dyn ClassFilter>>) -> Self {
        Self { filters }
    }
}

impl ClassFilter for OrFilter {
    fn matches(&self, class: &SmaliClass) -> bool {
        self.filters.iter().any(|f| f.matches(class))
    }
}

/// Helper functions to create method filters
pub mod method_filters {
    type SmaliMethodVec = Vec<Box<dyn Fn(&SmaliMethod) -> bool>>;

    use super::*;

    /// Create a filter for methods with a specific return type
    pub fn returns_type(return_type: TypeSignature) -> Box<dyn Fn(&SmaliMethod) -> bool> {
        Box::new(move |method: &SmaliMethod| method.signature.return_type == return_type)
    }

    /// Create a filter for methods with a specific name
    pub fn named(name: String) -> Box<dyn Fn(&SmaliMethod) -> bool> {
        Box::new(move |method: &SmaliMethod| method.name == name)
    }

    /// Create a filter for methods with a specific number of arguments
    pub fn with_arg_count(count: usize) -> Box<dyn Fn(&SmaliMethod) -> bool> {
        Box::new(move |method: &SmaliMethod| method.signature.args.len() == count)
    }

    /// Create a filter that combines multiple conditions with AND logic
    pub fn all_of(filters: SmaliMethodVec) -> Box<dyn Fn(&SmaliMethod) -> bool> {
        Box::new(move |method: &SmaliMethod| filters.iter().all(|f| f(method)))
    }

    /// Create a filter that combines multiple conditions with OR logic
    pub fn any_of(filters: SmaliMethodVec) -> Box<dyn Fn(&SmaliMethod) -> bool> {
        Box::new(move |method: &SmaliMethod| filters.iter().any(|f| f(method)))
    }
}

/// Helper functions to create common instruction patterns
pub mod instructions {
    use smali::types::SmaliInstruction;

    /// Create a simple pattern that returns a constant boolean value
    pub fn return_boolean(value: bool) -> Box<dyn Fn() -> Vec<SmaliInstruction>> {
        Box::new(move || {
            use SmaliInstruction::Instruction;

            let const_value = if value { "0x1" } else { "0x0" };

            vec![
                Instruction(format!("const/4 v0, {}", const_value)),
                Instruction("return v0".to_string()),
            ]
        })
    }

    /// Create a simple pattern that returns a constant int value
    pub fn return_int(value: i32) -> Box<dyn Fn() -> Vec<SmaliInstruction>> {
        Box::new(move || {
            use SmaliInstruction::Instruction;

            vec![
                Instruction(format!("const/4 v0, {:#x}", value)),
                Instruction("return v0".to_string()),
            ]
        })
    }

    /// Create a pattern that returns null
    pub fn return_null() -> Box<dyn Fn() -> Vec<SmaliInstruction>> {
        Box::new(|| {
            use SmaliInstruction::Instruction;

            vec![
                Instruction("const/4 v0, 0x0".to_string()),
                Instruction("return-object v0".to_string()),
            ]
        })
    }

    /// Create custom instructions
    pub fn custom<F>(generator: F) -> Box<dyn Fn() -> Vec<SmaliInstruction>>
    where
        F: Fn() -> Vec<SmaliInstruction> + 'static,
    {
        Box::new(generator)
    }
}
