use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, NoterError>;

#[derive(Error, Debug)]
pub enum NoterError {
    #[error("IO error: {0}")]
    Io(io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Invalid title: {0}")]
    InvalidTitle(String),

    #[error("Note not found: {0}")]
    NoteNotFound(i64),

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Editor not found")]
    EditorNotFound,

    #[error("Editor error: {0}")]
    EditorError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<io::Error> for NoterError {
    fn from(error: io::Error) -> Self {
        NoterError::Io(error)
    }
}

impl From<rusqlite::Error> for NoterError {
    fn from(err: rusqlite::Error) -> Self {
        NoterError::Database(err.to_string())
    }
}
