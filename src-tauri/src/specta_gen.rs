#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

use crate::commands;

pub fn builder() -> Builder<tauri::Wry> {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![commands::greet::greet]);

    #[cfg(debug_assertions)]
    builder
        .export(
            Typescript::default().header(
                r#"// These exports keep the generated file compatible with `noUnusedLocals`.
export { TAURI_CHANNEL };
export { __makeEvents__ };
"#,
            ),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    builder
}
