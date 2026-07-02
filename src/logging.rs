use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;

use crate::config::{LOG_DIRECTORY, LOG_FILE_NAME};

/// Routes all `tracing` events to `logs/simulation.log`.
///
/// Writing to a file keeps the alternate screen of the TUI clean.
/// The returned guard must stay alive until the program exits,
/// otherwise buffered log lines are lost.
pub fn init() -> WorkerGuard {
    let file_appender = rolling::never(LOG_DIRECTORY, LOG_FILE_NAME);
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .init();

    guard
}
