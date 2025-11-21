use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpatialError {
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Unknown spatial type: {0}")]
    UnknownType(u8),
}
