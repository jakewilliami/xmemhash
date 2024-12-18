//! Handle different archive types
//!
//! Different archive types ([`ArchiveType`]) require different handling ([`decompress`](crate::decompress))

use super::decompress::{gzip, sevenzip, tar, zip};
use std::{path::PathBuf, str::FromStr};
use strum::EnumIter;

#[derive(EnumIter)]
pub enum ArchiveType {
    Zip,
    SevenZip,
    Gzip,
    Tar,
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
            "application/gzip" => Ok(ArchiveType::Gzip),
            "application/x-tar" => Ok(ArchiveType::Tar),
            _ => Err(()),
        }
    }
}

impl From<ArchiveType> for String {
    fn from(archive_type: ArchiveType) -> Self {
        String::from(match archive_type {
            ArchiveType::Zip => "zip",
            ArchiveType::SevenZip => "7zip",
            ArchiveType::Gzip => "gzip",
            ArchiveType::Tar => "tar",
        })
    }
}

// Returns a vector of collections of bytes pertaining to each file
pub fn get_file_data_from_archive(path: &String, archive_type: ArchiveType) -> Vec<EnclosedFile> {
    match archive_type {
        ArchiveType::Zip => zip::get_files_from_zip_archive(path),
        ArchiveType::SevenZip => sevenzip::get_files_from_7z_archive(path),
        ArchiveType::Gzip => gzip::get_files_from_gzip_or_tarball(path),
        ArchiveType::Tar => tar::get_files_from_tar(path),
    }
}
