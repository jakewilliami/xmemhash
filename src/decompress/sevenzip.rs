use crate::archive::EnclosedFile;
use rpassword::prompt_password;
use sevenz_rust::{
    Error::{MaybeBadPassword, PasswordRequired},
    Password, SevenZReader,
};
use std::{
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
    process,
};

fn sz_archive_is_encrypted<R>(szr: &mut SevenZReader<R>) -> bool
where
    R: Seek,
    R: Read,
{
    // You have to try to read the file entries in order to check if you get
    // a password required error
    //
    // TODO: is there a better way to check if the file is encrypted?  E.g.
    //   https://docs.rs/crate/sevenz-rust/0.6.1/source/src/decoders.rs#142
    matches!(szr.for_each_entries(|_, _| Ok(true)), Err(PasswordRequired))
}

fn try_decrypt_from_7z_archive(path: &String) -> SevenZReader<File> {
    let password = prompt_password("Enter password: ").unwrap();
    let password = Password::from(password.as_str());

    SevenZReader::open(path, password).unwrap()
}

fn sz_archive_is_unencrypted<R>(szr: &mut SevenZReader<R>) -> bool
where
    R: Seek,
    R: Read,
{
    // You have to try to read the file contents within the archive before we know
    // if the password was (maybe) incorrect
    //
    // TODO: is there a better way to check if the file is still encrypted?
    !matches!(
        szr.for_each_entries(|_file, reader| {
            let mut content = Vec::new();
            let _ = reader.read_to_end(&mut content)?;
            Ok(true)
        }),
        Err(MaybeBadPassword(..))
    )
}

pub fn get_files_from_7z_archive(path: &String) -> Vec<EnclosedFile> {
    // Specify blank password if not password protected
    let password = Password::from("");
    let mut szr = SevenZReader::open(path, password).unwrap();

    // Request password from user if required
    if sz_archive_is_encrypted(&mut szr) {
        szr = try_decrypt_from_7z_archive(path);
        if !sz_archive_is_unencrypted(&mut szr) {
            // TODO: try three more times
            // TODO: Note that MaybeBadPassword could also be due to corrupted 7z file
            eprintln!("Incorrect password");
            process::exit(1);
        }
    }

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
    //
    // I was also initially concerned that I wouldn't be able to decrypt the file properly
    // because the documentation is not amazing and I couldn't see a way to do so after I
    // was getting an error for `UnsupportedCompressionMethod("AES256SHA256")`, but looking
    // through the source code, I found that support for AES256SHA256 is locked behind the
    // "aes256" feature.  I thought I'd have to do weird things with
    // `sevenz_rust::aes256sha256::AesEncoderOptions` but it's all good!
    szr.for_each_entries(|file, reader| {
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
