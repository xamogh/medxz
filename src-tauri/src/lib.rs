#![forbid(unsafe_code)]
#![warn(clippy::all)]

mod error;
mod logging;
mod specta_gen;

use crate::error::{AppError, AppResult};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_gen::builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            logging::init();
            tracing::info!("tauri app starting");
            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
#[specta::specta]
fn greet(name: String) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::EmptyName);
    }

    Ok(format!("Hello, {trimmed}! You've been greeted from Rust!"))
}
