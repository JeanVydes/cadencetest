use tracing::Level;

/// # Logging Subscriber
/// 
/// This module provides a function to start a logging subscriber using the `tracing` crate.
pub fn start_logging_subscriber(level: Level) {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
    
    tracing::info!("Logging subscriber started with level: {:?}", level);
}
