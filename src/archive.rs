use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};
use zip::{read::ZipFile, ZipArchive};

pub enum ArchiveType {
    Zip,
}

#[derive(Debug)]
pub struct EnclosedFile {
    path: Option<PathBuf>,
    pub bytes: Vec<u8>,
}

impl EnclosedFile {
    // Extract name from file path, or default to an empty string
    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|name| name.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string()
    }
}

impl FromStr for ArchiveType {
    type Err = ();

    fn from_str(input: &str) -> Result<ArchiveType, Self::Err> {
        match input {
            "application/zip" => Ok(ArchiveType::Zip),
            // "application/x-7z-compressed" => ArchiveType::7z,
            _ => Err(()),
        }
    }
}

// Read bytes from zip file contained within zip archive
fn get_bytes_from_file(file: &mut ZipFile) -> Vec<u8> {
    // TODO: read in parts so that the full file is never in memory
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content);
    content
}

// Returns a vector of collections of bytes pertaining to each file
pub fn get_file_data_from_archive(path: &String) -> Vec<EnclosedFile> {
    let path = Path::new(path);
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    let mut archive = ZipArchive::new(buf).unwrap();

    let mut files = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let bytes = get_bytes_from_file(&mut file);
        files.push(EnclosedFile {
            path: file.enclosed_name(),
            bytes,
        });
    }

    files
}
