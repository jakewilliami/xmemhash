use super::archive::EnclosedFile;
use sevenz_rust::{Password, SevenZReader};
use std::path::PathBuf;

pub fn get_file_from_7z_archive(path: &String) -> Vec<EnclosedFile> {
    let password = Password::from("");
    let mut szr = SevenZReader::open(path, password).unwrap();

    let mut files = Vec::new();

    // I could iterate over `szr.archive().files.iter()`, but I found the
    // `for_each_entries` function in this example:
    //   https://docs.rs/crate/sevenz-rust/latest/source/examples/decompress_progress.rs.
    //
    // Which I believe handles nested files, and gives me a reader rather than just
    // the archive entry
    //
    // When I was originally looking at the `sevenz-rust` crate, I was worried because
    // the API only looked like it supported decompressing *to disk* which is precisely
    // not what I want.  Even more concerning when it looks like the author deleted the
    // crate's source from GitHub!  So it's good to find a file reader directly in the API.
    let _ = szr
        .for_each_entries(|file, reader| {
            let mut content = Vec::new();
            let _ = reader.read_to_end(&mut content)?;
            let file_name = file.name.clone();
            files.push(EnclosedFile {
                path: Some(PathBuf::from(file_name)),
                bytes: content,
            });

            Ok(true)
        })
        .unwrap();

    files
}
