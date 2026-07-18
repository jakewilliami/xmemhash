//! Handle tar archive format
//!
//! Read archive files from tar files.  .tar.gz files are handled separately in [`gzip`](super::gzip).  NOTE: tar does not support encryption

use crate::archive::{ArchiveEntry, EntryData};
use std::{
    fs::File,
    io::{Cursor, Read},
};
use tar::Archive;

pub trait ReadTarArchive {
    fn get_files_from_tar(self) -> Vec<ArchiveEntry>;
}

impl<R: Read> ReadTarArchive for Archive<R> {
    fn get_files_from_tar(mut self) -> Vec<ArchiveEntry> {
        let mut files = Vec::new();

        // https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html#decompress-a-tarball-while-removing-a-prefix-from-the-paths
        self.entries()
            .unwrap()
            .filter_map(|e| e.ok())
            .for_each(|mut entry| {
                let path_buf = entry.path().ok().map(|p| p.into_owned());
                let data = if entry.header().entry_type().is_dir() {
                    EntryData::Directory(Vec::new())
                } else {
                    let mut bytes = Vec::new();
                    entry.read_to_end(&mut bytes).unwrap();
                    EntryData::File(bytes)
                };
                files.push(ArchiveEntry {
                    path: path_buf,
                    data,
                });
            });

        files
    }
}

pub fn get_files_from_tar(path: &String) -> Vec<ArchiveEntry> {
    let file = File::open(path).unwrap();
    Archive::new(file).get_files_from_tar()
}

// Returns a vector of archive entries pertaining to each file, from buffer
//
// This is used for recursing into nested archives
pub fn get_files_from_tar_bytes(bytes: Vec<u8>) -> Vec<ArchiveEntry> {
    Archive::new(Cursor::new(bytes)).get_files_from_tar()
}
