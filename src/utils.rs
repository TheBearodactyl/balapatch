use java_locator::*;
use std::fs::File;
use std::io::{Cursor, Read, Write, copy};
use std::path::PathBuf;
use std::{io::BufWriter, path::Path};
use which::which;

/// A struct that provides in-memory string I/O capabilities
///
/// Implements `Write` for building up content and provides
/// `reader()` to get a `Read` implementor for the content
pub struct StringBuf {
    buffer: Vec<u8>,
}

impl StringBuf {
    /// Creates a new empty StringIO buffer
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Creates a reader that provides read access to the current buffer contents
    ///
    /// The reader will read from the start of the buffer and cannot modify it
    pub fn reader(&self) -> impl Read + '_ {
        Cursor::new(&self.buffer)
    }

    /// Consumes the StringIO and returns the contents as a String
    ///
    /// # Errors
    /// Returns an error if the buffer contains invalid UTF-8
    pub fn into_string(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.buffer)
    }

    /// Returns the current contents as a String reference
    ///
    /// # Errors
    /// Returns an error if the buffer contains invalid UTF-8
    pub fn as_string(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.buffer)
    }
}

impl Write for StringBuf {
    /// Appends data to the buffer
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    /// No-op flush implementation
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Default for StringBuf {
    fn default() -> Self {
        Self::new()
    }
}

/// Downloads a file from a given URL and saves it to a specified file path.
///
/// * `url`: The URL to download the file from.
/// * `save_path`: The path where the file should be saved.
pub fn download_file(url: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let path = Path::new(save_path);
    let mut file = BufWriter::new(std::fs::File::create(path)?);
    let content = response.bytes()?;

    copy(&mut content.as_ref(), &mut file)?;

    Ok(())
}

/// Returns true if Java is installed and available on the system, false otherwise.
pub fn return_java_install() -> (bool, Option<PathBuf>) {
    match java_locator::locate_java_home() {
        Ok(java_home) => (
            true,
            Some(PathBuf::from(
                java_locator::locate_java_home().expect("Fuck"),
            )),
        ),
        Err(e) => {
            eprintln!("Java not found: {}", e);
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
