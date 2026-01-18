use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error("name cannot be empty")]
    EmptyName,
}

pub type AppResult<T> = Result<T, AppError>;
