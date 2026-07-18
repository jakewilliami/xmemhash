//! Handle gzip archive format
//!
//! Read archive files from gzip and tarball (.tar.gz) files.  Both types have the same MIME type, so we handle these together (though, see [`tar`](super::tar)).  NOTE: gzip does not support encryption

use super::tar::ReadTarArchive;
use crate::archive::{ArchiveEntry, EntryData};
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use tar::Archive;

fn to_gz_decoder(path: &String) -> GzDecoder<File> {
    let file = File::open(path).unwrap();
    GzDecoder::new(file)
}

fn to_archive(path: &String) -> Archive<GzDecoder<File>> {
    Archive::new(to_gz_decoder(path))
}

fn count_archive_files(path: &String) -> usize {
    let mut archive = to_archive(path);
    if let Ok(entries) = archive.entries() {
        entries.filter_map(|e| e.ok()).count()
    } else {
        0
    }
}

fn count_archive_entries(path: &String) -> usize {
    let mut archive = to_archive(path);
    if let Ok(entries) = archive.entries() {
        entries.count()
    } else {
        0
    }
}

// A file is an archive (.tar) file if it contains at least one valid entry
// The file is necessarily compressed (.gz) if we got to this function from archive.rs
fn is_tar_gz(path: &String) -> bool {
    !(count_archive_files(path) == 0 || count_archive_entries(path) == 0)
}

pub fn get_files_from_gzip_or_tarball(path: &String) -> Vec<ArchiveEntry> {
    if is_tar_gz(path) {
        to_archive(path).get_files_from_tar()
    } else {
        let mut gzd = to_gz_decoder(path);

        // We have to construct the file name based on given path because it has no
        // associated metadata in the gzip format
        let name = Path::new(path).file_stem().map(PathBuf::from);
        let mut bytes = Vec::new();
        gzd.read_to_end(&mut bytes).unwrap();

        // Gzip format has no support for multiple files, because it's only doing
        // compression, not archiving/containerising.  NB: as a result of this, gzip
        // by itself does not know anything about file structure, which is why we
        // have to construct the inner file based on the given path
        vec![ArchiveEntry {
            path: name,
            data: EntryData::File(bytes),
        }]
    }
}

// Utilities on bytes buffer
//
// This is used for recursing into nested archives

fn to_gz_decoder_from_bytes(bytes: &[u8]) -> GzDecoder<Cursor<&[u8]>> {
    GzDecoder::new(Cursor::new(bytes))
}

fn to_archive_from_bytes(bytes: &[u8]) -> Archive<GzDecoder<Cursor<&[u8]>>> {
    Archive::new(to_gz_decoder_from_bytes(bytes))
}

fn count_archive_files_from_bytes(bytes: &[u8]) -> usize {
    let mut archive = to_archive_from_bytes(bytes);
    if let Ok(entries) = archive.entries() {
        entries.filter_map(|e| e.ok()).count()
    } else {
        0
    }
}

fn count_archive_entries_from_bytes(bytes: &[u8]) -> usize {
    let mut archive = to_archive_from_bytes(bytes);
    if let Ok(entries) = archive.entries() {
        entries.count()
    } else {
        0
    }
}

fn is_tar_gz_bytes(bytes: &[u8]) -> bool {
    !(count_archive_files_from_bytes(bytes) == 0 || count_archive_entries_from_bytes(bytes) == 0)
}

// Variant of `get_files_from_gzip_or_tarball` but for reading from a buffer.
// `context` is used only to recover a file name from plain (non-tarball) gzip content,
// since raw gzip has no filename field of its own to fall back on.
//
// This is used for recursing into nested archives
pub fn get_files_from_gzip_or_tarball_bytes(bytes: Vec<u8>, context: &Path) -> Vec<ArchiveEntry> {
    if is_tar_gz_bytes(&bytes) {
        to_archive_from_bytes(&bytes).get_files_from_tar()
    } else {
        let mut gzd = to_gz_decoder_from_bytes(&bytes);

        let name = context.file_stem().map(PathBuf::from);
        let mut content = Vec::new();
        gzd.read_to_end(&mut content).unwrap();

        vec![ArchiveEntry {
            path: name,
            data: EntryData::File(content),
        }]
    }
}
