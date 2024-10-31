//! Specify the hashing algorithm to use for hash computation
//!
//! Currently supports SHA256, SHA1, and MD5.

use clap::ValueEnum;

#[derive(ValueEnum, Clone)]
pub enum HashAlgo {
    Md5,
    Sha1,
    Sha256,
}
