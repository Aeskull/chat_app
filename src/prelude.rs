use std::error::Error;
use std::fmt::Display;

/// Generic Result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct ConnectionError {
    message: String
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