//! Handle file type inference
//!
//! Infer type of archive, if valid ([`ArchiveType`]), and associated error reporting

use super::archive::ArchiveType;
use std::{path::Path, str::FromStr};

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
                    PathValid::invalid("unsupported archive type")
                }
            } else {
                PathValid::invalid("file is not a valid archive")
            }
        } else {
            // Cannot get file type, so we must assume the input was not an archive
            PathValid::invalid("cannot get file type")
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
