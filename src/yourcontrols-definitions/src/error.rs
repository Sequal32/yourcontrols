use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error opening file!")]
    FileError(#[from] std::io::Error),
    #[error("Error deserializing YAML!")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Error deserializing YAML from file path {1}!")]
    YamlFileError(serde_yaml::Error, String),
    // Parsing
    #[error("Missing field {0}")]
    MissingField(&'static str),
    #[error("Missing something!")]
    None,
}
