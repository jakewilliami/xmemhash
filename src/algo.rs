use clap::ValueEnum;

#[derive(ValueEnum, Clone)]
pub enum HashAlgo {
    Md5,
    Sha1,
    Sha256,
}
