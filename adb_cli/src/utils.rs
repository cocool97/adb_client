use env_logger::{Builder, Env};

/// Sets up appropriate logger level:
/// - if `RUST_LOG` environment variable is set, use its value
/// - else, use `debug` CLI option
pub fn setup_logger(debug: bool) {
    Builder::from_env(Env::default().default_filter_or(if debug { "debug" } else { "info" }))
        .init();
}
