//! Handle tar archive format
//!
//! Read archive files from tar files.  .tar.gz files are handled separately in [`gzip`](super::gzip).  NOTE: tar does not support encryption

use crate::archive::EnclosedFile;
use std::{fs::File, io::Read};
use tar::Archive;

pub trait ReadTarArchive {
    fn get_files_from_tar(self) -> Vec<EnclosedFile>;
}

impl<R: Read> ReadTarArchive for Archive<R> {
    fn get_files_from_tar(mut self) -> Vec<EnclosedFile> {
        let mut files = Vec::new();

        // https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html#decompress-a-tarball-while-removing-a-prefix-from-the-paths
        self.entries()
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
}

pub fn get_files_from_tar(path: &String) -> Vec<EnclosedFile> {
    let file = File::open(path).unwrap();
    Archive::new(file).get_files_from_tar()
}
