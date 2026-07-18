//! Handle 7zip archive format
//!
//! Read archive files from (potentially password-protected) 7zip files

use crate::archive::{ArchiveEntry, EntryData};
use rpassword::prompt_password;
use sevenz_rust::{Error::PasswordRequired, Password, SevenZReader};
use std::{
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
    process,
};

fn sz_archive_is_unencrypted<R>(szr: &mut SevenZReader<R>) -> bool
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

fn try_decrypt_from_7z_archive(path: &String) -> SevenZReader<File> {
    let password = prompt_password("Enter password: ").unwrap();
    let password = Password::from(password.as_str());

    let mut szr = match SevenZReader::open(path, password) {
        Ok(szr) => szr,
        Err(_) => {
            eprintln!("[ERROR] Incorrect password");
            process::exit(1);
        }
    };

    if !sz_archive_is_unencrypted(&mut szr) {
        eprintln!("[ERROR] Incorrect password");
        process::exit(1);
    }

    szr
}

pub fn get_files_from_7z_archive(path: &String) -> Vec<ArchiveEntry> {
    // TODO: try three more times if password did not work?
    // TODO: Note that MaybeBadPassword could also be due to corrupted 7z files
    let mut szr = match SevenZReader::open(path, Password::from("")) {
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

    let mut files = Vec::new();

    // I could iterate over `szr.archive().files.iter()`, but I found the
    // `for_each_entries` function in this example:
    //   https://docs.rs/crate/sevenz-rust/latest/source/examples/decompress_progress.rs.
    //
    // Which I believe handles nested files, and gives me a reader rather than just
    // the archive entry.
    //
    // When I was originally looking at the `sevenz-rust` crate, I was worried because
    // the API only looked like it supported decompressing *to disk* which is precisely
    // not what I want.  Even more concerning when it looks like the author deleted the
    // crate's source from GitHub!  So it's good to find a file reader directly in the API.
    //
    // I was also initially concerned that I wouldn't be able to decrypt the file properly
    // because the documentation is not amazing and I couldn't see a way to do so after I
    // was getting an error for `UnsupportedCompressionMethod("AES256SHA256")`, but looking
    // through the source code, I found that support for AES256SHA256 is locked behind the
    // "aes256" feature.  I thought I'd have to do weird things with
    // `sevenz_rust::aes256sha256::AesEncoderOptions` but it's all good!
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
