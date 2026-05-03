use clap::{ArgAction, Parser, crate_authors, crate_name, crate_version};
use std::{path::PathBuf, process};
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

    let mut table = Table::new("{:>}  {:<}");
    let archive_type = file::archive_type(&cli.file_path);
    let recurse = if cli.no_recurse {
        0
    } else {
        cli.recurse.min(2)
    };

    if recurse > 1 {
        todo!("recursion into archives not yet supported")
    }

    // Construct hash table from contents of archives
    //
    // TODO: Since v1.6.0–v1.6.1, we handle recursion options better.  There is still some
    // way to go till an ideal output (see #8).  We do not yet support "extra" recursion
    // into nested archives.
    let entries = archive::get_file_data_from_archive(&cli.file_path, archive_type);
    add_entries_to_table(&mut table, &entries, &cli.hash, recurse, None);

    print!("{}", table);

    process::exit(0);
}

fn add_entries_to_table(
    table: &mut Table,
    entries: &[ArchiveEntry],
    algo: &HashAlgo,
    recurse: u8,
    prefix: Option<&str>,
) {
    for entry in entries {
        let name = match prefix {
            Some(p) => PathBuf::from(p).join(entry.name()),
            None => PathBuf::from(entry.name()),
        };
        let name = name.to_string_lossy().to_string();

        match &entry.data {
            EntryData::File(bytes) => {
                let hash = hash::get_hash_from_data(bytes, algo);
                table.add_row(row!(hash, name));
            }
            EntryData::Directory(_) if entry.is_empty_directory() => {
                table.add_row(row!(String::from("<empty directory>"), name));
            }
            EntryData::Directory(children) => match recurse {
                0 => {
                    table.add_row(row!(String::from("<directory>"), name));
                }
                _ => {
                    add_entries_to_table(table, children, algo, recurse, Some(&name));
                }
            },
        }
    }
}
