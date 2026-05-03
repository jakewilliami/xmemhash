use clap::{ArgAction, Parser, crate_authors, crate_name, crate_version};
use std::{path::Path, process};
use tabular::{Table, row};

mod algo;
mod archive;
mod decompress;
mod file;
mod hash;
mod tree;

use algo::HashAlgo;
use archive::{ArchiveEntry, EntryData};

#[derive(Parser)]
#[command(
    name = crate_name!(),
    author = crate_authors!(", "),
    version = crate_version!(),
)]
/// Extract archive in memory and get its contents' hash(es)
struct Cli {
    /// File path to archive to extract
    #[arg(
        action = ArgAction::Set,
        num_args = 1,
        value_name = "file path",
    )]
    file_path: String,

    /// The hashing algorithm to use for the resulting hash
    #[clap(value_enum)]
    #[arg(
        long = "hash",
        action = ArgAction::Set,
        num_args = 0..=1,
        value_name = "hashing algorithm",
        default_value_t = HashAlgo::Sha256,
    )]
    hash: HashAlgo,

    /// Recurse into substructures
    ///
    /// Recurse is set to 1 (`-r`) by default, in order to recurse into subdirectories.  Use `--no-recurse` to disable this.  Set this to 2 (`-rr`) if you want to recurse into nested archives (not yet supported).
    #[arg(
        long = "recurse",
        short = 'r',
        action = ArgAction::Count,
        default_value_t = 1,
    )]
    recurse: u8,

    /// Do not recurse into substructures
    ///
    /// Ignore `--recurse`/`-r` option (which is on by default) and actively do not recurse
    #[arg(
        long = "no-recurse",
        action = ArgAction::SetTrue,
        conflicts_with = "recurse",
        default_value_t = false,
    )]
    no_recurse: bool,

    /// Display output as a tree
    #[arg(
        long = "tree",
        action = ArgAction::SetTrue,
        default_value_t = false,
    )]
    tree: bool,
}

fn main() {
    let cli = Cli::parse();

    if !file::path_is_valid(&cli.file_path) {
        eprintln!(
            "[ERROR] File is not a valid input: {}",
            file::path_invalid_reason(&cli.file_path)
        );
        process::exit(1);
    }

    let archive_type = file::archive_type(&cli.file_path);
    let recurse = if cli.no_recurse {
        0
    } else {
        cli.recurse.min(2)
    };

    if recurse > 1 {
        todo!("recursion into archives not yet supported")
    }

    let entries = archive::get_file_data_from_archive(&cli.file_path, archive_type);

    // Construct hash table from contents of archives
    //
    // TODO: Since v1.6.0/v1.7.0, we handle recursion options better (see #8).  We do not
    // yet support "extra" recursion into nested archives.
    if cli.tree {
        print_tree(
            &entries,
            &cli.hash,
            recurse,
            Path::new(""),
            Some(&cli.file_path),
        );
    } else {
        let mut table = Table::new("{:>}  {:<}");
        add_entries_to_table(&mut table, &entries, &cli.hash, recurse, Path::new(""));
        print!("{}", table);
    }

    process::exit(0);
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
            EntryData::Directory(_) if entry.is_empty_directory() => {
                table.add_row(row!(String::from("<empty directory>"), name_str));
            }
            EntryData::Directory(children) => match recurse {
                0 => {
                    table.add_row(row!(String::from("<directory>"), name_str));
                }
                1 => {
                    add_entries_to_table(table, children, algo, recurse, &name);
                }
                _ => {
                    add_entries_to_table(table, children, algo, recurse, &name);
                }
            },
        }
    }
}

fn print_tree(
    entries: &[ArchiveEntry],
    algo: &HashAlgo,
    recurse: u8,
    prefix: &Path,
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
                println!("{}{}{}  {}", prefix.display(), connector, hash, name);
            }
            EntryData::Directory(_) if entry.is_empty_directory() => {
                println!("{}{}{}/  <empty>", prefix.display(), connector, name);
            }
            EntryData::Directory(children) => match recurse {
                0 => {
                    println!("{}{}{}/  <directory>", prefix.display(), connector, name);
                }
                _ => {
                    println!("{}{}{}/", prefix.display(), connector, name);
                    print_tree(children, algo, recurse, &prefix.join(child_prefix), None);
                }
            },
        }
    }
}
