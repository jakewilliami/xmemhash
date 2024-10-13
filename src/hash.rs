use super::algo::HashAlgo;
use super::archive::EnclosedFile;
use digest::Digest;
use sha1::Sha1;
use sha2::Sha256;

// https://stackoverflow.com/q/64326373/
fn compute_hash<D: Digest>(data: &Vec<u8>) -> String
where
    D::OutputSize: std::ops::Add,
    <D::OutputSize as std::ops::Add>::Output: digest::generic_array::ArrayLength<u8>,
{
    let mut hasher = D::new();
    hasher.update(data);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

fn get_hash_from_data(data: &Vec<u8>, algo: &HashAlgo) -> String {
    match algo {
        HashAlgo::Sha1 => compute_hash::<Sha1>(data),
        HashAlgo::Sha256 => compute_hash::<Sha256>(data),
    }
}

pub fn get_hash_from_enclosed_file(file: &EnclosedFile, algo: &HashAlgo) -> String {
    get_hash_from_data(&file.bytes, algo)
}
