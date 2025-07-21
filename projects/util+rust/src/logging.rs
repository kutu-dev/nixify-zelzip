//! Logging management.

use tracing_subscriber::fmt;

/// Setup logging with the `tracing` crate, tailored for CLI applications.
pub fn setup_logging_for_cli() {
    let format = fmt::format().without_time();
    tracing_subscriber::fmt().event_format(format).init();
}
