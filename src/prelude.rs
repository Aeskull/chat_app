use std::error::Error;
use std::fmt::Display;

/// Generic Result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct MyError {
    content: String
}

impl MyError {
    pub fn new(s: &str) -> Self {
        Self {
            content: s.to_owned(),
        }
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.content)
    }
}

impl Error for MyError {}