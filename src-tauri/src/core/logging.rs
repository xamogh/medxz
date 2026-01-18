use std::sync::Once;

use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(default_log_level()));

        let _ = fmt()
            .with_env_filter(filter)
            .with_ansi(cfg!(debug_assertions))
            .with_target(false)
            .try_init();
    });
}

fn default_log_level() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    }
}
