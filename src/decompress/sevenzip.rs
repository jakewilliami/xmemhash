//! Handle 7zip archive format
//!
//! Read archive files from (potentially password-protected) 7zip files

use crate::archive::{ArchiveEntry, EntryData};
use rpassword::prompt_password;
use sevenz_rust2::{ArchiveReader, Error::PasswordRequired, Password};
use std::{
    fs::File,
    io::{Cursor, Read, Seek},
    path::{Path, PathBuf},
    process,
};

fn sz_archive_is_unencrypted<R>(szr: &mut ArchiveReader<R>) -> bool
where
    R: Seek + Read,
{
    // You have to try to read the file contents within the archive before we know
    // if the password was (maybe) incorrect
    //
    // TODO: is there a better way to check if the file is still encrypted?  E.g.,
    //   https://docs.rs/crate/sevenz-rust/0.6.1/source/src/decoders.rs#142
    szr.for_each_entries(|_file, reader| {
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;
        Ok(true)
    })
    .is_ok()
}

// Given an opened and decrypted ArchiveReader over any source that implements
// Read and Seek, walk ever entry and collate its contents into a flat Vec<ArchiveEntry>
fn get_files_from_7z_reader<R>(mut szr: ArchiveReader<R>) -> Vec<ArchiveEntry>
where
    R: Read + Seek,
{
    let mut files = Vec::new();

    szr.for_each_entries(|file, reader| {
        let file_name = file.name.clone();
        let data = if file.is_directory() {
            EntryData::Directory(Vec::new())
        } else {
            let mut bytes = Vec::new();
            let _ = reader.read_to_end(&mut bytes)?;
            EntryData::File(bytes)
        };
        files.push(ArchiveEntry {
            path: Some(PathBuf::from(file_name)),
            data,
        });

        Ok(true)
    })
    .unwrap();

    files
}

fn try_decrypt_from_7z_archive(path: &String) -> ArchiveReader<File> {
    let password = prompt_password("Enter password: ").unwrap();
    let password = Password::from(password.as_str());

    let mut szr = match ArchiveReader::open(path, password) {
        Ok(szr) => szr,
        Err(_) => {
            eprintln!("[ERROR] Incorrect password");
            process::exit(1);
        }
    };

    // Confirm that we have successfully decrypted the archive; that is, all of its
    // contents are readable
    if !sz_archive_is_unencrypted(&mut szr) {
        eprintln!("[ERROR] Incorrect password");
        process::exit(1);
    }

    szr
}

// Variant of `try_decrypt_from_7z_archive` on buffer, not path
//
// This is used for recursing into nested archives
fn try_decrypt_from_7z_bytes<'a>(
    bytes: &'a [u8],
    context: &'a Path,
) -> ArchiveReader<Cursor<&'a [u8]>> {
    let password = prompt_password(format!("Enter password ({}): ", context.display())).unwrap();
    let password = Password::from(password.as_str());

    let mut szr = match ArchiveReader::new(Cursor::new(bytes), password) {
        Ok(szr) => szr,
        Err(_) => {
            eprintln!("[ERROR] Incorrect password");
            process::exit(1);
        }
    };

    // Confirm that we have successfully decrypted the archive; that is, all of its
    // contents are readable
    if !sz_archive_is_unencrypted(&mut szr) {
        eprintln!("[ERROR] Incorrect password");
        process::exit(1);
    }

    szr
}

pub fn get_files_from_7z_archive(path: &String) -> Vec<ArchiveEntry> {
    // 7z files can be fully encrypted, or its file contents are encrypted but not the
    // header.  If the former, then `ArchiveReader::new` will fail.  Even if we seemingly
    // open the archive, we need to confirm that all of its contents are readable.
    //
    // TODO: try three more times if password did not work?
    // TODO: Note that MaybeBadPassword could also be due to corrupted 7z files
    let szr = match ArchiveReader::open(path, Password::from("")) {
        Ok(mut szr) => {
            if sz_archive_is_unencrypted(&mut szr) {
                szr
            } else {
                try_decrypt_from_7z_archive(path)
            }
        }
        Err(PasswordRequired) => try_decrypt_from_7z_archive(path),
        Err(e) => {
            eprintln!("[ERROR] Failed to open archive: {e}");
            process::exit(1);
        }
    };

    get_files_from_7z_reader(szr)
}

// Variant of `get_files_from_7z_archive` on buffer, not path
//
// This is used for recursing into nested archives
pub fn get_files_from_7z_bytes(
    bytes: Vec<u8>,
    allow_nested_encryption: bool,
    context: &Path,
) -> Result<Vec<ArchiveEntry>, Vec<u8>> {
    let szr = match ArchiveReader::new(Cursor::new(bytes.as_slice()), Password::from("")) {
        Ok(mut szr) => {
            if sz_archive_is_unencrypted(&mut szr) {
                szr
            } else if allow_nested_encryption {
                try_decrypt_from_7z_bytes(&bytes, context)
            } else {
                return Err(bytes);
            }
        }
        Err(PasswordRequired) => {
            if allow_nested_encryption {
                try_decrypt_from_7z_bytes(&bytes, context)
            } else {
                return Err(bytes);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to open nested archive: {e}");
            process::exit(1);
        }
    };

    Ok(get_files_from_7z_reader(szr))
}
