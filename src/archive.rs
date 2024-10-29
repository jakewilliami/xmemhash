use super::decompress::sevenzip;
use super::decompress::zip;
use std::{path::PathBuf, str::FromStr};

pub enum ArchiveType {
    Zip,
    SevenZip,
}

pub struct EnclosedFile {
    pub path: Option<PathBuf>,
    pub bytes: Vec<u8>,
}

impl EnclosedFile {
    // Extract name from file path, or default to an empty string
    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|name| name.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string()
    }
}

impl FromStr for ArchiveType {
    type Err = ();

    fn from_str(input: &str) -> Result<ArchiveType, Self::Err> {
        match input {
            "application/zip" => Ok(ArchiveType::Zip),
            "application/x-7z-compressed" => Ok(ArchiveType::SevenZip),
            _ => Err(()),
        }
    }
}

// Returns a vector of collections of bytes pertaining to each file
pub fn get_file_data_from_archive(path: &String, archive_type: ArchiveType) -> Vec<EnclosedFile> {
    match archive_type {
        ArchiveType::Zip => zip::get_files_from_zip_archive(path),
        ArchiveType::SevenZip => sevenzip::get_files_from_7z_archive(path),
    }
}
