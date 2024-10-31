//! Handle gzip archive format
//!
//! Read archive files from gzip and tarball (.tar.gz) files.  Both types have the same MIME type, so we handle these together (though, see [`tar`](super::tar)).  NOTE: gzip does not support encryption

use super::tar::ReadTarArchive;
use crate::archive::EnclosedFile;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tar::Archive;

trait GzipUtils {
    fn to_gz_decoder(&self) -> GzDecoder<File>;
    fn to_archive(&self) -> Archive<GzDecoder<File>>;
}

impl GzipUtils for String {
    fn to_gz_decoder(&self) -> GzDecoder<File> {
        let file = File::open(self).unwrap();
        GzDecoder::new(file)
    }

    fn to_archive(&self) -> Archive<GzDecoder<File>> {
        let gzd = self.to_gz_decoder();
        Archive::new(gzd)
    }
}

trait CountArchiveElements {
    fn count_archive_files(&self) -> usize;
    fn count_archive_entries(&self) -> usize;
}

impl CountArchiveElements for String {
    fn count_archive_files(&self) -> usize {
        let mut archive = self.to_archive();
        if let Ok(entries) = archive.entries() {
            entries.filter_map(|e| e.ok()).count()
        } else {
            0
        }
    }

    fn count_archive_entries(&self) -> usize {
        let mut archive = self.to_archive();
        if let Ok(entries) = archive.entries() {
            entries.count()
        } else {
            0
        }
    }
}

trait IsTarball {
    fn is_tar_gz(&self) -> bool;
}

impl IsTarball for String {
    fn is_tar_gz(&self) -> bool {
        // A file is an archive (.tar) file if it contains at least one valid entry
        // The file is necessarily compressed (.gz) if we got to this function from archive.rs
        !(self.count_archive_files() == 0 || self.count_archive_entries() == 0)
    }
}

pub fn get_files_from_gzip_or_tarball(path: &String) -> Vec<EnclosedFile> {
    if path.is_tar_gz() {
        path.to_archive().get_files_from_tar()
    } else {
        let mut gzd = path.to_gz_decoder();

        // We have to construct the file name based on given path because it has no associated metadata in the gzip format
        let path = Path::new(path).file_stem().map(PathBuf::from);
        let mut bytes = Vec::new();
        let _ = &gzd.read_to_end(&mut bytes).unwrap();

        // Gzip format has no support for multiple files, because it's only doing compression, not archiving/containerising.  NB: as a result of this, gzip by itself does not know anything about file structure, which is why we have to construct the inner file based on the given path
        vec![EnclosedFile { path, bytes }]
    }
}
