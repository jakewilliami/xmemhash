//! Handle file type inference
//!
//! Infer type of archive, if valid ([`ArchiveType`]), and associated error reporting

use super::archive::ArchiveType;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};
use strum::IntoEnumIterator;

struct PathValid {
    is_valid: bool,
    reason: Option<String>,
}

impl PathValid {
    fn valid() -> Self {
        Self {
            is_valid: true,
            reason: None,
        }
    }

    fn invalid(reason: &str) -> Self {
        Self {
            is_valid: false,
            reason: Some(String::from(reason)),
        }
    }
}

impl From<&String> for PathValid {
    fn from(path: &String) -> Self {
        let path = Path::new(&path);

        // Check that path exists
        if !path.exists() {
            return PathValid::invalid("path does not exist");
        }

        // Check that the file provided is indeed a supported archive
        let kind = infer::get_from_path(path).expect("file should be readable");

        if let Some(kind) = kind {
            // Valid file to extract if the matcher type is an archive
            if kind.matcher_type() == infer::MatcherType::Archive {
                if ArchiveType::from_str(kind.mime_type()).is_ok() {
                    PathValid::valid()
                } else {
                    let supported_archive_types = ArchiveType::iter()
                        .map(String::from)
                        .collect::<Vec<_>>()
                        .join(", ");
                    PathValid::invalid(&format!(
                        "unsupported archive type \"{}\"; supported types: {}",
                        kind.mime_type(),
                        supported_archive_types,
                    ))
                }
            } else {
                PathValid::invalid(&format!("invalid file type \"{}\"", kind.mime_type()))
            }
        } else {
            // Cannot get file type, so we must assume the input was not an archive
            if path.is_plain_text() {
                PathValid::invalid("file is plain text")
            } else {
                PathValid::invalid("cannot get file type")
            }
        }
    }
}

pub fn path_is_valid(path: &String) -> bool {
    PathValid::from(path).is_valid
}

pub fn path_invalid_reason(path: &String) -> String {
    let path_valid = PathValid::from(path);

    if path_valid.is_valid {
        String::new()
    } else {
        path_valid
            .reason
            .expect("invalid path must have associated reason")
    }
}

// Get type of archive from file path
//
// Assumes path is valid and file is an archive type!  use path_is_valid to confirm
pub fn archive_type(path: &String) -> ArchiveType {
    let kind = infer::get_from_path(path)
        .expect("file should be readable")
        .expect("file type should be obtainable");
    ArchiveType::from_str(kind.mime_type()).unwrap()
}

trait FileTypeInference {
    fn is_plain_text(&self) -> bool;
}

impl FileTypeInference for Path {
    // Plain text, but not necessarily ASCII
    fn is_plain_text(&self) -> bool {
        let file = File::open(self).unwrap();
        let reader = BufReader::new(file);

        // Try reading the file as UTF-8 directly
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Check each character for being printable or whitespace
                    if !line.chars().all(|c| c.is_whitespace() || c.is_printable()) {
                        return false;
                    }
                }
                Err(_) => {
                    // Non-UTF-8 sequence encountered
                    return false;
                }
            }
        }

        true
    }
}

trait CharExt {
    fn is_printable(&self) -> bool;
}

impl CharExt for char {
    fn is_printable(&self) -> bool {
        // Allow whitespace but disallow other control chars
        !self.is_control() || self.is_whitespace()
    }
}
