//! Handle tar archive format
//!
//! Read archive files from tar files.  .tar.gz files are handled separately in [`gzip`](super::gzip).  NOTE: tar does not support encryption

use crate::archive::EnclosedFile;
use std::{fs::File, io::Read};
use tar::Archive;

pub fn get_files_from_tar(path: &String) -> Vec<EnclosedFile> {
    let file = File::open(path).unwrap();
    let mut archive = Archive::new(file);

    let mut files = Vec::new();

    archive
        .entries()
        .unwrap()
        .filter_map(|e| e.ok())
        .for_each(|mut entry| {
            let path_buf = entry.path().ok().map(|p| p.into_owned());
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            files.push(EnclosedFile {
                path: path_buf,
                bytes,
            });
        });

    files
}
