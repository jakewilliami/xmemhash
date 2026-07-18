//! Handle different archive types
//!
//! Different archive types ([`ArchiveType`]) require different handling ([`decompress`](crate::decompress))

use crate::{
    decompress::{gzip, rar, sevenzip, tar, zip},
    tree,
};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use strum::EnumIter;

#[derive(EnumIter, Clone, Copy)]
pub enum ArchiveType {
    Zip,
    SevenZip,
    Gzip,
    Tar,
    Rar,
}

pub enum EntryData {
    File(Vec<u8>),
    Directory(Vec<ArchiveEntry>),
    // The expanded contents of a nested archive found during recursion (see `recurse.rs`).
    // This is distinct from `Directory` so output formatting can tell this apart from a
    // subdirectory.
    NestedArchive(Vec<ArchiveEntry>),
}

pub struct ArchiveEntry {
    pub path: Option<PathBuf>,
    pub data: EntryData,
}

impl ArchiveEntry {
    // Extract name from file path, or default to an empty string
    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn is_empty_directory(&self) -> bool {
        matches!(
            &self.data,
            EntryData::Directory(children) | EntryData::NestedArchive(children)
            if children.is_empty()
        )
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
            "application/vnd.rar" => Ok(ArchiveType::Rar),
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
            ArchiveType::Rar => "rar",
        })
    }
}

// Returns a vector of archive entries pertaining to each file
pub fn get_file_data_from_archive(path: &String, archive_type: ArchiveType) -> Vec<ArchiveEntry> {
    let flat = match archive_type {
        ArchiveType::Zip => zip::get_files_from_zip_archive(path),
        ArchiveType::SevenZip => sevenzip::get_files_from_7z_archive(path),
        ArchiveType::Gzip => gzip::get_files_from_gzip_or_tarball(path),
        ArchiveType::Tar => tar::get_files_from_tar(path),
        ArchiveType::Rar => rar::get_files_from_rar_archive(path),
    };
    tree::build_tree(flat)
}

// Returns a vector of archive entries pertaining to each file, from a buffer
//
// This is used for recursion into nested archives.  If we are not able to recurse, we return
// the bytes of the file as an error value so that we can still compute its hash.
pub fn get_file_data_from_bytes(
    bytes: Vec<u8>,
    archive_type: ArchiveType,
    allow_nested_encryption: bool,
    context: &Path,
) -> Result<Vec<ArchiveEntry>, Vec<u8>> {
    let flat = match archive_type {
        ArchiveType::Zip => zip::get_files_from_zip_bytes(bytes, allow_nested_encryption, context),
        ArchiveType::SevenZip => {
            sevenzip::get_files_from_7z_bytes(bytes, allow_nested_encryption, context)
        }
        ArchiveType::Gzip => Ok(gzip::get_files_from_gzip_or_tarball_bytes(bytes, context)),
        ArchiveType::Tar => Ok(tar::get_files_from_tar_bytes(bytes)),
        ArchiveType::Rar => rar::get_files_from_rar_bytes(bytes, allow_nested_encryption, context),
    }?;
    Ok(tree::build_tree(flat))
}
