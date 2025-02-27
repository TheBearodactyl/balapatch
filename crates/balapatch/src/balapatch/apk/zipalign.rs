use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone)]
pub struct ZipAlign {
    apk_path: PathBuf,
    output_dir: Option<PathBuf>,
    alignment: u64,
}

impl ZipAlign {
    pub fn new(apk_path: PathBuf, output_dir: Option<PathBuf>, alignment: u64) -> Self {
        Self {
            apk_path,
            output_dir,
            alignment,
        }
    }

    pub fn align(&self, force: bool) -> anyhow::Result<()> {
        let output_path = self.output_dir.clone().unwrap();
        let input_path = self.apk_path.clone();

        if !force && std::path::Path::new(output_path.as_path()).exists() {
            return Err(anyhow!(
                "Output file '{}' exists",
                output_path.as_path().to_str().unwrap()
            ));
        }

        let input_file = File::open(input_path)?;
        let mut input_zip = ZipArchive::new(input_file)?;

        let output_file = File::create(output_path)?;
        let mut output_zip = ZipWriter::new(output_file);
        let mut current_offset = 0u64;

        for i in 0..input_zip.len() {
            let mut entry = input_zip.by_index(i)?;
            let name = entry.name().to_string();
            let compression_method = entry.compression();

            let filename_bytes = name.as_bytes();
            let filename_len = filename_bytes.len();

            // Calculate base header size (30 bytes) + filename length
            let base_header_size = 30u64;
            let data_start = current_offset + base_header_size + filename_len as u64;

            // Calculate required padding
            let padding = (self.alignment - (data_start % self.alignment)) % self.alignment;

            // Prepare data for the extra field (data size + padding bytes)
            let data_size = padding as u16; // Note: potential truncation for large alignments
            let mut data_part = Vec::new();
            data_part.extend_from_slice(&data_size.to_le_bytes());
            data_part.resize(data_part.len() + padding as usize, 0);
            let data_box = data_part.into_boxed_slice();

            // Build file options with correct extra data
            let mut options = FileOptions::default()
                .compression_method(compression_method)
                .unix_permissions(entry.unix_mode().unwrap_or(0o644));

            options.add_extra_data(0x0000, data_box, true)?;

            // Write entry header
            output_zip.start_file(name, options)?;

            // Update offset tracking
            current_offset += base_header_size
				+ filename_len as u64
				+ 2 * 2 // 2 bytes for header ID, 2 bytes for data size in extra field
				+ padding
				+ entry.compressed_size();

            // Copy entry contents
            copy(&mut entry, &mut output_zip)?;
        }

        output_zip.finish()?;
        Ok(())
    }

    pub fn verify_zip(&self, verbose: bool) -> Result<bool> {
        let file = File::open(self.apk_path.as_path())?;
        let mut zip = ZipArchive::new(file)?;
        let mut found_bad = false;

        for i in 0..zip.len() {
            let entry = zip.by_index(i)?;
            let data_start = entry.data_start();

            if entry.compression() == CompressionMethod::Stored {
                if data_start % self.alignment as u64 != 0 {
                    if verbose {
                        println!(
                            "{:8} {} (BAD - mod {} = {})",
                            data_start,
                            entry.name(),
                            self.alignment,
                            data_start % self.alignment as u64
                        );
                    }
                    found_bad = true;
                } else if verbose {
                    println!("{:8} {} (OK)", data_start, entry.name());
                }
            } else if verbose {
                println!("{:8} {} (OK - compressed)", data_start, entry.name());
            }
        }

        Ok(!found_bad)
    }
}

// Verification function remains unchanged
