use crate::{
    algo::HashAlgo,
    archive::{ArchiveEntry, EntryData},
    hash,
};
use std::path::Path;
use tabular::{Table, row};

pub fn print_table(entries: &[ArchiveEntry], algo: &HashAlgo, recurse: u8) {
    let mut table = Table::new("{:>}  {:<}");
    add_entries_to_table(&mut table, entries, algo, recurse, Path::new(""));
    print!("{}", table);
}

fn add_entries_to_table(
    table: &mut Table,
    entries: &[ArchiveEntry],
    algo: &HashAlgo,
    recurse: u8,
    prefix: &Path,
) {
    for entry in entries {
        let name = prefix.join(entry.name());
        let name_str = name.to_string_lossy().to_string();

        match &entry.data {
            EntryData::File(bytes) => {
                let hash = hash::get_hash_from_data(bytes, algo);
                table.add_row(row!(hash, name_str));
            }
            EntryData::Directory(_) | EntryData::NestedArchive(_) if entry.is_empty_directory() => {
                table.add_row(row!(String::from("<empty directory>"), name_str));
            }
            EntryData::Directory(children) => match recurse {
                0 => {
                    table.add_row(row!(String::from("<directory>"), name_str));
                }
                _ => {
                    add_entries_to_table(table, children, algo, recurse, &name);
                }
            },
            EntryData::NestedArchive(children) => {
                add_entries_to_table(table, children, algo, recurse, &name);
            }
        }
    }
}

pub fn print_tree(entries: &[ArchiveEntry], algo: &HashAlgo, recurse: u8, label: Option<&str>) {
    print_tree_inner(entries, algo, recurse, "", label)
}

fn print_tree_inner(
    entries: &[ArchiveEntry],
    algo: &HashAlgo,
    recurse: u8,
    prefix: &str,
    label: Option<&str>,
) {
    if let Some(label) = label {
        println!("{}", label);
    }

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let name = entry.name();

        match &entry.data {
            EntryData::File(bytes) => {
                let hash = hash::get_hash_from_data(bytes, algo);
                println!("{}{}{}  {}", prefix, connector, hash, name);
            }
            EntryData::Directory(_) | EntryData::NestedArchive(_) if entry.is_empty_directory() => {
                println!("{}{}{}/  <empty>", prefix, connector, name);
            }
            EntryData::Directory(children) => match recurse {
                0 => {
                    println!("{}{}{}/  <directory>", prefix, connector, name);
                }
                _ => {
                    println!("{}{}{}/", prefix, connector, name);
                    let child_prefix = format!("{}{}", prefix, child_prefix);
                    print_tree_inner(children, algo, recurse, &child_prefix, None);
                }
            },
            EntryData::NestedArchive(children) => {
                println!("{}{}{}", prefix, connector, name);
                let child_prefix = format!("{}{}", prefix, child_prefix);
                print_tree_inner(children, algo, recurse, &child_prefix, None);
            }
        }
    }
}
