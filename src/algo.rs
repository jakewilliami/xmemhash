use clap::ValueEnum;

#[derive(ValueEnum, Clone)]
pub enum HashAlgo {
    Sha256,
}
