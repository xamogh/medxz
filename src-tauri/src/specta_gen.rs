#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

use crate::commands;

pub fn builder() -> Builder<tauri::Wry> {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::greet::greet,
        commands::auth::login,
        commands::auth::me,
        commands::auth::logout
    ]);

    #[cfg(debug_assertions)]
    if let Err(err) = builder.export(
        Typescript::default().header(
            r#"// These exports keep the generated file compatible with `noUnusedLocals`.
export { TAURI_CHANNEL };
export { __makeEvents__ };
"#,
        ),
        "../src/bindings.ts",
    ) {
        eprintln!("warning: failed to export TypeScript bindings: {err}");
    }

    builder
}
