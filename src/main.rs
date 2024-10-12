use clap::{crate_authors, crate_name, crate_version, ArgAction, Parser};
use std::process;
use tabular::{row, Table};

mod algo;
mod archive;
mod decompress;
mod file;
mod hash;

use algo::HashAlgo;

// TODO:
//   - Support gz and/or tar?
//   - Add support for sha1 and md5 hashes
//   - Consider using something like compress-tools or archive-reader or unzip-rs instead of handling separate archive formats myself.  See also tools like ouch-org/ouch
//   - Support nested folders

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
    for file in archive::get_file_data_from_archive(&cli.file_path, archive_type) {
        let name = file.name();
        let hash = hash::get_hash_from_enclosed_file(&file, &cli.hash);
        table.add_row(row!(hash, name));
    }
    print!("{}", table);

    process::exit(0);
}
