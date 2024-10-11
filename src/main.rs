use clap::{crate_authors, crate_name, crate_version, ArgAction, Parser};

mod algo;

use algo::HashAlgo;

// TODO:
//   - Ensure reading file is piecewise so that the full inner contents of the archive never exists in memory, only parts of it
//   - Handle multiple inner files
//   - Support zip
//   - Support 7z
//   - Support password-protected archives
//   - Support gz and/or tar?

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
        short = 'h',
        long = "hash",
        action = ArgAction::Set,
        num_args = 0..=1,
        value_name = "hashing algorithm",
        default_value_t = HashAlgo::Sha256,
    )]
    hash: HashAlgo,
}

fn main() {
    // let cli = Cli::parse();
    todo!()
}
