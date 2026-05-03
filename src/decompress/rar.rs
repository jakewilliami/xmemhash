//! Handle rar archive format
//!
//! Read archive files from (potentially password-protected) rar files

use crate::archive::{ArchiveEntry, EntryData};
use rpassword::prompt_password;
use std::process;
use unrar::{
    error::{Code, UnrarError},
    {Archive, CursorBeforeHeader, OpenArchive, Process},
};

type RarArchive = OpenArchive<Process, CursorBeforeHeader>;

// The problem is, RAR supports both fully encrypted files, or files whose listings
// or header information are available but whose file contents are not.
//   - Fully encrypted: github.com/muja/unrar.rs/raw/0628d12b/data/comment-hpw-password.rar
//   - Partially encrypted: github.com/muja/unrar.rs/raw/0628d12b/data/crypted.rar
//   - Expected behaviour: github.com/muja/unrar.rs/blob/0628d12b/tests/crypted.rs
fn is_encrypted(path: &String) -> bool {
    let arc = Archive::new(path).open_for_processing().unwrap();
    match arc.read_header() {
        // Case 1: the content and headers are fully encrypted
        Err(e) if e.code == Code::MissingPassword => true,
        Ok(Some(header)) => {
            // We may be able to open the header but not read the contents as it may
            // only be partially encrypted
            matches!(header.read(), Err(e) if e.code == Code::MissingPassword)
        }
        _ => false,
    }
}

fn try_decrypt_from_rar_archive(path: &String) -> RarArchive {
    let password = prompt_password("Enter password: ").unwrap();
    Archive::with_password(path, &password)
        .open_for_processing()
        .unwrap()
}

fn handle_rar_error(e: UnrarError, path: &String, prompted: bool) -> RarArchive {
    match e.code {
        Code::MissingPassword if !prompted => try_decrypt_from_rar_archive(path),
        Code::MissingPassword | Code::BadPassword => {
            eprintln!("[ERROR] Incorrect password");
            process::exit(1);
        }
        Code::BadData => {
            if is_encrypted(path) {
                eprintln!("[ERROR] Incorrect password");
            } else {
                eprintln!("[ERROR] Error reading RAR archive: {}", e);
            }
            process::exit(1);
        }
        _ => {
            eprintln!("[ERROR] Error reading RAR archive: {}", e);
            process::exit(1);
        }
    }
}

pub fn get_files_from_rar_archive(path: &String) -> Vec<ArchiveEntry> {
    let mut files = Vec::new();
    let mut archive = Archive::new(path).open_for_processing().unwrap();
    let mut prompted = false;

    loop {
        match archive.read_header() {
            Err(e) => {
                archive = handle_rar_error(e, path, prompted);
                prompted = true;
            }
            Ok(None) => break,
            Ok(Some(header)) => {
                let is_dir = header.entry().is_directory();
                let filename = header.entry().filename.clone();

                if is_dir {
                    archive = header.skip().unwrap();
                    files.push(ArchiveEntry {
                        path: Some(filename),
                        data: EntryData::Directory(Vec::new()),
                    });
                } else {
                    match header.read() {
                        Err(e) => {
                            archive = handle_rar_error(e, path, prompted);
                            prompted = true;
                        }
                        Ok((bytes, rest)) => {
                            archive = rest;
                            files.push(ArchiveEntry {
                                path: Some(filename),
                                data: EntryData::File(bytes),
                            });
                        }
                    }
                }
            }
        }
    }

    files
}
