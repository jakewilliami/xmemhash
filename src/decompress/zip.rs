//! Handle zip archive format
//!
//! Read archive files from (potentially password-protected) zip files

use crate::archive::{ArchiveEntry, EntryData};
use rpassword::prompt_password;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    path::Path,
    process,
};
use zip::{
    ZipArchive,
    read::ZipFile,
    result::{ZipError, ZipResult},
};

fn zip_archive_is_encrypted<R>(archive: &mut ZipArchive<R>) -> bool
where
    R: Seek + Read,
{
    // Logic taken from:
    //   https://github.com/zip-rs/zip2/blob/6d394564/src/result.rs#L38-L48
    //
    // TODO: is there a better way to check if the file is encrypted?
    matches!(
        archive.by_index(0),
        Err(ZipError::UnsupportedArchive(ZipError::PASSWORD_REQUIRED))
    )
}

fn try_decrypt_from_zip_archive_index<'a, R>(
    archive: &'a mut ZipArchive<R>,
    i: usize,
    context: Option<&Path>,
) -> ZipResult<ZipFile<'a>>
where
    R: Seek + Read,
{
    let prompt = match context {
        Some(ctx) => format!("Enter password ({}): ", ctx.display()),
        None => "Enter password: ".to_string(),
    };
    let password = prompt_password(prompt).unwrap();
    let password = password.as_bytes();

    archive.by_index_decrypt(i, password)
}

fn get_files_from_zip_archive_index<'a, R>(
    archive: &'a mut ZipArchive<R>,
    i: usize,
    context: Option<&Path>,
) -> ZipFile<'a>
where
    R: Seek + Read,
{
    if !zip_archive_is_encrypted(archive) {
        archive.by_index(i).unwrap()
    } else if let Ok(file) = try_decrypt_from_zip_archive_index(archive, i, context) {
        file
    } else {
        // TODO: try 3 more times before giving up.  I am having issues with mutable lifetimes
        // of the archive object so I am just trying once for now
        eprintln!("[ERROR] Incorrect password");
        process::exit(1);
    }
}

// Read bytes from zip file contained within zip archive
fn get_bytes_from_zip_file(file: &mut ZipFile) -> Vec<u8> {
    // TODO: read in parts so that the full file is never in memory
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content);
    content
}

// Given an opened ZipArchive over any source that implements Read and Seek,
// walk ever entry and collate its contents into a flat Vec<ArchiveEntry>
fn get_files_from_zip_archive_reader<R>(
    mut archive: ZipArchive<R>,
    context: Option<&Path>,
) -> Vec<ArchiveEntry>
where
    R: Read + Seek,
{
    let mut files = Vec::new();
    for i in 0..archive.len() {
        let mut file = get_files_from_zip_archive_index(&mut archive, i, context);
        let data = if file.is_dir() {
            EntryData::Directory(Vec::new())
        } else {
            EntryData::File(get_bytes_from_zip_file(&mut file))
        };
        files.push(ArchiveEntry {
            path: file.enclosed_name(),
            data,
        });
    }
    files
}

// Returns a vector of archive entries pertaining to each file, from path
pub fn get_files_from_zip_archive(path: &String) -> Vec<ArchiveEntry> {
    let file = File::open(Path::new(path)).unwrap();
    let buf = BufReader::new(file);
    let archive = ZipArchive::new(buf).unwrap();
    get_files_from_zip_archive_reader(archive, None)
}

// Returns a vector of archive entries pertaining to each file, from buffer
//
// This is used for recursing into nested archives
pub fn get_files_from_zip_bytes(
    bytes: Vec<u8>,
    allow_nested_encryption: bool,
    context: &Path,
) -> Result<Vec<ArchiveEntry>, Vec<u8>> {
    let cursor = Cursor::new(bytes.as_slice());
    let mut archive = ZipArchive::new(cursor).unwrap();

    if zip_archive_is_encrypted(&mut archive) && !allow_nested_encryption {
        return Err(bytes);
    }

    Ok(get_files_from_zip_archive_reader(archive, Some(context)))
}
