use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error("name cannot be empty")]
    EmptyName,

    #[error("invalid server url: {message}")]
    InvalidServerUrl { message: String },

    #[error("network error: {message}")]
    Network { message: String },

    #[error("keychain error: {message}")]
    Keychain { message: String },

    #[error("server error {status} {code}: {message}")]
    ServerError {
        status: u16,
        code: String,
        message: String,
    },
}

pub type AppResult<T> = Result<T, AppError>;
