pub fn setup_logger(debug: bool) {
    // RUST_LOG variable has more priority then "--debug" flag
    if std::env::var("RUST_LOG").is_err() {
        let level = match debug {
            true => "trace",
            false => "info",
        };

        std::env::set_var("RUST_LOG", level);
    }

    // Setting default log level as "info" if not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
}
