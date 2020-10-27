use std::{fmt::Display};

pub enum LoadError {
    FileError(std::io::Error),
    ParseError(serde_yaml::Error)
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileError(e) => write!(f, "Could not read file: {}", e),
            LoadError::ParseError(e) => write!(f, "Could not parse file: {}", e)
        }
    }
}