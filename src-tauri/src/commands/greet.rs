use crate::core::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub(crate) fn greet(name: String) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::EmptyName);
    }

    Ok(format!("Hello, {trimmed}! You've been greeted from Rust!"))
}
