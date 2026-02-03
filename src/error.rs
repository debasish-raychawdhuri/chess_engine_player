use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Engine error: {0}")]
    Engine(String),
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Engine(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Engine(s)
    }
}
