use super::algo::HashAlgo;
use super::archive::EnclosedFile;
use sha2::{Digest, Sha256};

fn get_hash_from_data(data: &Vec<u8>, algo: &HashAlgo) -> String {
    let mut hasher = match algo {
        HashAlgo::Sha256 => Sha256::new(),
    };

    hasher.update(data);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

pub fn get_hash_from_enclosed_file(file: &EnclosedFile, algo: &HashAlgo) -> String {
    get_hash_from_data(&file.bytes, algo)
}
