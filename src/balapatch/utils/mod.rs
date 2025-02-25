use crate::balapatch::tui::progress::GLOBAL_MP;
use indicatif::ProgressBar;
use java_locator::*;
use std::fmt::Debug;
use std::fs::File;
use std::io::{copy, Cursor, Read, Write};
use std::path::PathBuf;
use std::{io::BufWriter, path::Path};
use tokio_stream::StreamExt;
use tracing::error;
use which::which;

pub mod string_buf;
pub mod writer;

/// Downloads a file from a given URL and saves it to a specified file path.
///
/// * `url`: The URL to download the file from.
/// * `save_path`: The path where the file should be saved.
pub async fn download_file(url: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pb = GLOBAL_MP.add(ProgressBar::new_spinner());
    pb.set_message("Downloading file...");

    let response = reqwest::get(url).await?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = crate::balapatch::tui::progress::create_bytes_progress("Downloading", total_size);

    let path = Path::new(save_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = BufWriter::new(File::create(path)?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Download complete!");
    Ok(())
}

/// Returns true if Java is installed and available on the system, false otherwise.
pub fn return_java_install() -> (bool, Option<PathBuf>) {
    match locate_java_home() {
        Ok(java_home) => (true, Some(PathBuf::from(locate_java_home().expect("Fuck")))),
        Err(e) => {
            error!("Java not found: {}", e);
            (false, None)
        }
    }
}

/// Returns true if all required dependencies are installed, false otherwise.
pub fn check_for_dependencies() -> bool {
    let mut found_deps: i8 = 0;

    if return_java_install().0 && return_java_install().1.unwrap().exists() {
        found_deps += 1;
    }

    if which("zipalign").is_ok() {
        found_deps += 1;
    }

    if which("apktool").is_ok() {
        found_deps += 1;
    }

    found_deps == 3
}

/// Returns the contents of the specified file as a string.
///
/// * `file_path`: The path to the file to read.
pub fn get_file_contents(file_path: &str) -> String {
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    contents
}

#[derive(Debug, Clone)]
pub struct Either<T, U> {
    val: EitherVariant<T, U>,
}

#[derive(Debug, Clone)]
enum EitherVariant<T, U> {
    Left(T),
    Right(U),
}

impl<T, U> Either<T, U> {
    pub fn new_left(value: T) -> Self {
        Self {
            val: EitherVariant::Left(value),
        }
    }

    pub fn new_right(value: U) -> Self {
        Self {
            val: EitherVariant::Right(value),
        }
    }

    pub fn is_left(&self) -> bool {
        matches!(self.val, EitherVariant::Left(_))
    }

    pub fn is_right(&self) -> bool {
        matches!(self.val, EitherVariant::Right(_))
    }

    pub fn get_left(self) -> Option<T> {
        match self.val {
            EitherVariant::Left(value) => Some(value),
            EitherVariant::Right(_) => None,
        }
    }

    pub fn get_right(self) -> Option<U> {
        match self.val {
            EitherVariant::Left(_) => None,
            EitherVariant::Right(value) => Some(value),
        }
    }

    pub fn into_left(self) -> Option<T> {
        match self.val {
            EitherVariant::Left(value) => Some(value),
            EitherVariant::Right(_) => None,
        }
    }

    pub fn into_right(self) -> Option<U> {
        match self.val {
            EitherVariant::Left(_) => None,
            EitherVariant::Right(value) => Some(value),
        }
    }

    pub fn into_result(self) -> Result<T, U> {
        match self.val {
            EitherVariant::Left(value) => Ok(value),
            EitherVariant::Right(value) => Err(value),
        }
    }

    pub fn into_result_reverse(self) -> Result<U, T> {
        match self.val {
            EitherVariant::Left(value) => Err(value),
            EitherVariant::Right(value) => Ok(value),
        }
    }
}

impl<T, U: Default> From<Option<T>> for Either<T, U> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => Either::new_left(v),
            None => Either::new_right(U::default()),
        }
    }
}

impl<T, U> From<Result<T, U>> for Either<T, U> {
    fn from(res: Result<T, U>) -> Self {
        match res {
            Ok(v) => Either::new_left(v),
            Err(e) => Either::new_right(e),
        }
    }
}
