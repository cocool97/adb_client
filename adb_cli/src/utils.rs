/// # Safety
///
/// This conditionally mutates the process' environment.
/// See [`std::env::set_var`] for more info.
pub unsafe fn setup_logger(debug: bool) {
    // RUST_LOG variable has more priority then "--debug" flag
    if std::env::var("RUST_LOG").is_err() {
        let level = match debug {
            true => "trace",
            false => "info",
        };

        unsafe { std::env::set_var("RUST_LOG", level) };
    }

    env_logger::init();
}
