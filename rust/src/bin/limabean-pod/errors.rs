use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum Error {
    Unexpected(Box<dyn std::error::Error>),
    JsonDecode(serde_json::Error, String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match &self {
            Unexpected(msg) => write!(f, "unexpected error {}", &msg),
            JsonDecode(e, input) => write!(f, "JSON decode error: {}\n{}", &e, &input),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Unexpected(Box::new(value))
    }
}
