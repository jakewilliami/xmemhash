use crate::archive::EnclosedFile;
use rpassword::prompt_password;
use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
    process,
};
use zip::{
    read::ZipFile,
    result::{ZipError, ZipResult},
    ZipArchive,
};

fn zip_archive_is_encrypted<R>(archive: &mut ZipArchive<R>) -> bool
where
    R: Seek,
    R: Read,
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

fn try_decrypt_from_zip_archive_index<R>(
    archive: &mut ZipArchive<R>,
    i: usize,
) -> ZipResult<ZipFile>
where
    R: Seek,
    R: Read,
{
    let password = prompt_password("Enter password: ").unwrap();
    let password = password.as_bytes();

    archive.by_index_decrypt(i, password)
}

fn get_files_from_zip_archive_index<R>(archive: &mut ZipArchive<R>, i: usize) -> ZipFile
where
    R: Seek,
    R: Read,
{
    if !zip_archive_is_encrypted(archive) {
        archive.by_index(i).unwrap()
    } else if let Ok(file) = try_decrypt_from_zip_archive_index(archive, i) {
        file
    } else {
        // TODO: try 3 more times before giving up.  I am having issues with mutable lifetimes of the archive object so I am just trying once for now
        eprintln!("Incorrect password");
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

// Returns a vector of collections of bytes pertaining to each file
pub fn get_files_from_zip_archive(path: &String) -> Vec<EnclosedFile> {
    let path = Path::new(path);
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    let mut archive = ZipArchive::new(buf).unwrap();

    let mut files = Vec::new();
    for i in 0..archive.len() {
        let mut file = get_files_from_zip_archive_index(&mut archive, i);
        let bytes = get_bytes_from_zip_file(&mut file);
        files.push(EnclosedFile {
            path: file.enclosed_name(),
            bytes,
        });
    }

    files
}
