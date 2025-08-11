use std::fmt;

#[derive(Debug, Clone)]
pub enum StoreError {
    IoError(String),
    SerializationError(String),
    NotFound,
    InvalidKey,
    InvalidValue,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::IoError(msg) => write!(f, "IO error: {}", msg),
            StoreError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StoreError::NotFound => write!(f, "Key not found"),
            StoreError::InvalidKey => write!(f, "Invalid key"),
            StoreError::InvalidValue => write!(f, "Invalid value"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        StoreError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        StoreError::SerializationError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;