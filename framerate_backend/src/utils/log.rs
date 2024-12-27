use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::{fmt::writer::MakeWriterExt, layer::SubscriberExt};

pub fn setup_logger() {
    let debug_file = rolling::daily("logs", "info").with_max_level(Level::INFO);
    let warn_file = rolling::daily("logs", "warnings").with_max_level(Level::WARN);
    let all_files = debug_file.and(warn_file);

    let stdout_log =
        tracing_subscriber::fmt::layer().with_writer(std::io::stdout.with_max_level(Level::INFO));

    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(all_files)
        .with_ansi(false);

    let subscriber = tracing_subscriber::registry()
        .with(stdout_log)
        .with(file_log);

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
}
