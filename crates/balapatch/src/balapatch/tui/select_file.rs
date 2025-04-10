use std::io::ErrorKind;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use inquire::{Autocomplete, CustomUserError, InquireError, Text, autocompletion::Replacement};

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    input: String,
    paths: Vec<String>,
}

impl FilePathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input && !self.paths.is_empty() {
            return Ok(());
        }

        self.input = input.to_owned();
        self.paths.clear();

        let input_path = std::path::PathBuf::from(input);
        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    std::path::PathBuf::from("../../../../..")
                } else {
                    p.to_owned()
                }
            })
            .unwrap_or_else(|| std::path::PathBuf::from("../../../../.."));

        let scan_dir = if input.ends_with('/') {
            input_path
        } else {
            fallback_parent.clone()
        };

        let entries = match std::fs::read_dir(scan_dir) {
            Ok(read_dir) => Ok(read_dir),
            Err(err) if err.kind() == ErrorKind::NotFound => std::fs::read_dir(fallback_parent),
            Err(err) => Err(err),
        }?
        .collect::<Result<Vec<_>, _>>()?;

        for entry in entries {
            let path = entry.path();
            let path_str = if path.is_dir() {
                #[cfg(not(windows))]
                let path_separator = "/";

                #[cfg(windows)]
                let path_separator = "\\";

                format!("{}{}", path.to_string_lossy(), path_separator)
            } else {
                path.to_string_lossy().to_string()
            };

            self.paths.push(path_str);
        }

        Ok(())
    }

    fn fuzzy_sort(&self, input: &str) -> Vec<(String, i64)> {
        let mut matches: Vec<(String, i64)> = self
            .paths
            .iter()
            .filter_map(|path| {
                SkimMatcherV2::default()
                    .smart_case()
                    .fuzzy_match(path, input)
                    .map(|score| (path.clone(), score))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;

        let matches = self.fuzzy_sort(input);
        Ok(matches.into_iter().take(15).map(|(path, _)| path).collect())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;

        Ok(if let Some(suggestion) = highlighted_suggestion {
            Some(suggestion)
        } else {
            let matches = self.fuzzy_sort(input);
            matches
                .first()
                .map(|(path, _)| Some(path.clone()))
                .unwrap_or(None)
        })
    }
}

/// Shows a prompt for selecting a path.
/// Comes with autocomplete for file paths.
///
/// * `prompt_msg`: The prompt message to show when requesting the selection.
pub fn select_path_from_current_dir(prompt_msg: &str) -> Result<String, InquireError> {
    let current_dir = std::env::current_dir().expect("fuck");
    let help_message = format!("Current Directory: {}", current_dir.to_string_lossy());

    let ans = Text::new(prompt_msg)
        .with_autocomplete(FilePathCompleter::default())
        .with_help_message(&help_message)
        .prompt();

    ans
}
