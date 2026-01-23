use std::{fmt::Display, path::PathBuf};

#[derive(Debug)]
pub(crate) enum Error {
    FatalAndAlreadyExplained,
    Unexpected(Box<dyn std::error::Error>),
    CannotReadFile(PathBuf, std::io::Error),
    JsonDecode(serde_json::Error, String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match &self {
            FatalAndAlreadyExplained => Ok(()),
            Unexpected(msg) => write!(f, "unexpected error {}", &msg),
            CannotReadFile(path, e) => {
                write!(f, "cannot read file {}: {}", path.to_string_lossy(), e)
            }
            JsonDecode(e, input) => write!(f, "JSON decode error: {}\n{}", &e, &input),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Unexpected(Box::new(value))
    }
}
