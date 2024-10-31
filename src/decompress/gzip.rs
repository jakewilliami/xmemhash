use crate::archive::EnclosedFile;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tar::Archive;

trait ArchiveUtils {
    fn to_gz_decoder(&self) -> GzDecoder<File>;
    fn to_archive(&self) -> Archive<GzDecoder<File>>;
    fn count_archive_files(&self) -> usize;
    fn count_archive_entries(&self) -> usize;
    fn is_tar_gz(&self) -> bool;
}

impl ArchiveUtils for String {
    fn to_gz_decoder(&self) -> GzDecoder<File> {
        let file = File::open(self).unwrap();
        GzDecoder::new(file)
    }

    fn to_archive(&self) -> Archive<GzDecoder<File>> {
        let gzd = self.to_gz_decoder();
        Archive::new(gzd)
    }

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

    fn is_tar_gz(&self) -> bool {
        // A file is an archive (.tar) file if it contains at least one valid entry
        // The file is necessarily compressed (.gz) if we got to this function from archive.rs
        !(self.count_archive_files() == 0 || self.count_archive_entries() == 0)
    }
}

pub fn get_files_from_gzip_or_tarball(path: &String) -> Vec<EnclosedFile> {
    let mut files = Vec::new();

    if path.is_tar_gz() {
        // https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html#decompress-a-tarball-while-removing-a-prefix-from-the-paths
        let mut archive = path.to_archive();
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
    } else {
        let mut gzd = path.to_gz_decoder();

        // We have to construct the file name based on given path because it has no associated metadata in the gzip format
        let path = Path::new(path).file_stem().map(PathBuf::from);
        let mut bytes = Vec::new();
        let _ = &gzd.read_to_end(&mut bytes).unwrap();
        files.push(EnclosedFile { path, bytes });
    }

    files
}
