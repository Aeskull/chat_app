use std::{error::Error, fmt::Display};

/// Generic Result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// An error for connections. All it contains is a String dennoting the error (may expand later).
#[derive(Debug)]
pub struct ConnectionError {
    message: String,
}

impl ConnectionError {
    pub fn new(s: &str) -> Self {
        Self {
            message: s.to_owned(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)
    }
}

impl Error for ConnectionError {}
