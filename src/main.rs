mod algo;
mod archive;
mod decompress;
mod display;
mod file;
mod hash;
mod recurse;
mod tree;

use algo::HashAlgo;
use clap::{ArgAction, Parser, crate_authors, crate_name, crate_version};
use std::{
    env,
    io::{self, IsTerminal},
    path::Path,
    process,
};

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
    /// Recurse is set to 1 (`-r`) by default, in order to recurse into subdirectories.  Use `--no-recurse` to disable this.  Set this to 2 (`-rr`) if you want to recurse into nested archives.  If this is set to 3 (`-rrr`), this will even decrypt nested archives if they are encrypted, but this will ask for a password for every encrypted nested archive it finds.
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

    let is_terminal = io::stdout().is_terminal();
    if is_set("NO_COLOR") || is_set("NO_COLOUR") || !is_terminal {
        colored::control::set_override(false);
    }

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
        cli.recurse.min(3)
    };

    let entries = archive::get_file_data_from_archive(&cli.file_path, archive_type);
    let entries = if recurse > 1 {
        recurse::expand_nested_archives(entries, recurse, Path::new(&cli.file_path))
    } else {
        entries
    };

    // Construct hash table from contents of archives
    if cli.tree {
        display::print_tree(&entries, &cli.hash, recurse, Some(&cli.file_path));
    } else {
        display::print_table(&entries, &cli.hash, recurse);
    }

    process::exit(0);
}

// Stolen from gl:
//   <github.com/jakewilliami/gl/blob/9bd3fa96/src/env.rs#L1-L10>
fn is_set(var: &str) -> bool {
    let val = env::var(var);

    // Value must be set and non-empty
    if let Ok(val) = val {
        !val.is_empty()
    } else {
        false
    }
}
