use std::str::FromStr;

pub enum ArchiveType {
    Zip,
}

impl FromStr for ArchiveType {
    type Err = ();

    fn from_str(input: &str) -> Result<ArchiveType, Self::Err> {
        match input {
            "application/zip" => Ok(ArchiveType::Zip),
            _ => Err(()),
        }
    }
}
