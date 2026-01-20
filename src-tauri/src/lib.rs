#![forbid(unsafe_code)]
#![warn(clippy::all)]

mod commands;
mod core;
mod specta_gen;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_gen::builder();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            core::logging::init();
            tracing::info!("tauri app starting");
            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("error while running tauri application: {err}");
    }
}
