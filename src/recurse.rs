use crate::{
    archive::{self, ArchiveEntry, ArchiveType, EntryData},
    file,
};
use std::path::Path;

pub fn expand_nested_archives(
    entries: Vec<ArchiveEntry>,
    recurse_level: u8,
    context: &Path,
) -> Vec<ArchiveEntry> {
    let allow_nested_encryption = recurse_level >= 3;

    entries
        .into_iter()
        .map(|entry| match entry.data {
            EntryData::File(bytes) => {
                let Some(archive_type) = file::archive_type_from_bytes(&bytes) else {
                    return ArchiveEntry {
                        path: entry.path,
                        data: EntryData::File(bytes),
                    };
                };

                let entry_context = match &entry.path {
                    Some(p) => context.join(p),
                    None => context.to_path_buf(),
                };

                match archive::get_file_data_from_bytes(
                    bytes,
                    archive_type,
                    allow_nested_encryption,
                    &entry_context,
                ) {
                    Ok(nested) => ArchiveEntry {
                        path: entry.path,
                        data: EntryData::NestedArchive(expand_nested_archives(
                            nested,
                            recurse_level,
                            &entry_context,
                        )),
                    },
                    Err(bytes) => {
                        let reason = if matches!(archive_type, ArchiveType::Rar) {
                            "RAR recursion unsupported without writing to disk"
                        } else {
                            "incorrect, missing, or disallowed password"
                        };
                        eprintln!(
                            "[WARN] Skipping nested archive ({reason}): {}",
                            entry_context.display()
                        );
                        ArchiveEntry {
                            path: entry.path,
                            data: EntryData::File(bytes),
                        }
                    }
                }
            }
            EntryData::Directory(children) => ArchiveEntry {
                path: entry.path,
                data: EntryData::Directory(expand_nested_archives(
                    children,
                    recurse_level,
                    context,
                )),
            },
            EntryData::NestedArchive(children) => ArchiveEntry {
                path: entry.path,
                data: EntryData::NestedArchive(expand_nested_archives(
                    children,
                    recurse_level,
                    context,
                )),
            },
        })
        .collect()
}
