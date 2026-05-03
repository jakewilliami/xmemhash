use clap::{ArgAction, Parser, crate_authors, crate_name, crate_version};
use std::process;
use tabular::{Table, row};

mod algo;
mod archive;
mod decompress;
mod file;
mod hash;

use algo::HashAlgo;

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

    // Construct hash table from contents of archives
    for file in archive::get_file_data_from_archive(&cli.file_path, archive_type) {
        let name = file.name();

        // TODO: Since v1.6.0, we handle recursion options better.  There is still
        // some way to go till an ideal output (see #8).  We do not yet support
        // "extra" recursion into nested archives.
        match recurse {
            // Case 1: recurse is turned off.  We can log directories but only compute
            // the hash of files in the root directory
            0 => match hash::get_hash_from_archive_entry(&file, &cli.hash) {
                Some(hash) if !file.is_in_subdirectory() => {
                    table.add_row(row!(hash, name));
                }
                None => {
                    table.add_row(row!(String::from("<directory>"), name));
                }
                // Nested files are intentionally skipped when recursion is disabled.
                // This is the only case left unhandled, so we can do nothing
                _ => {}
            },
            // Case 2: we can recurse.  Because we print the full path of the files, we
            // can safely ignore directories
            1 => {
                // TODO: we will ignore directories that have no files inside.  We should
                // do something about that?
                if let Some(hash) = hash::get_hash_from_archive_entry(&file, &cli.hash) {
                    table.add_row(row!(hash, name));
                }
            }
            // Case 3: we will recurse into nested archives, but we have not yet implemented
            // this
            2 => {
                todo!("recursion into archives not yet supported")
            }
            // Fallback case: we only support recursion levels 0–2, no further
            _ => unreachable!(),
        }
    }

    print!("{}", table);

    process::exit(0);
}
