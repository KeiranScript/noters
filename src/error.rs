use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, NoterError>;

#[derive(Error, Debug)]
pub enum NoterError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

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

    EditorNotFound,
    EditorError(String),
}

impl From<rusqlite::Error> for NoterError {
    fn from(err: rusqlite::Error) -> Self {
        NoterError::Database(err.to_string())
    }
}
