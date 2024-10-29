use crate::archive::EnclosedFile;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tar::Archive;

fn into_gz_decoder(path: &String) -> GzDecoder<File> {
    let file = File::open(path).unwrap();
    GzDecoder::new(file)
}

fn into_archive(path: &String) -> Archive<GzDecoder<File>> {
    let gzd = into_gz_decoder(path);
    Archive::new(gzd)
}

fn count_archive_files(path: &String) -> usize {
    let mut archive = into_archive(path);
    // TODO: better unwrap
    let entries = archive.entries().unwrap();
    entries.filter_map(|e| e.ok()).count()
}

fn count_archive_entries(path: &String) -> usize {
    let mut archive = into_archive(path);
    // TODO: better unwrap
    archive.entries().unwrap().count()
}

fn is_tar_gz(path: &String) -> bool {
    if count_archive_files(path) == 0 || count_archive_entries(path) == 0 {
        return false;
    }

    let mut archive = into_archive(path);
    archive.entries().is_ok()
}

pub fn get_files_from_tarball(path: &String) -> Vec<EnclosedFile> {
    let mut files = Vec::new();

    if is_tar_gz(path) {
        // https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html#decompress-a-tarball-while-removing-a-prefix-from-the-paths
        let mut archive = into_archive(path);
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
        // If `Archive::new` failed then the file was just a .gz, not .tar.gz
        let mut gzd = into_gz_decoder(path);

        // We have to construct the file name based on given path because it has no associated metadata in the gzip format
        let path = Path::new(path).file_stem().map(PathBuf::from);
        let mut bytes = Vec::new();
        let _ = &gzd.read_to_end(&mut bytes).unwrap();
        files.push(EnclosedFile { path, bytes });
    }

    files
}
