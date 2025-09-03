mod conversion;
mod errors;
mod runtime;
mod types;

#[cfg(feature = "erlang")]
mod erlang;

#[cfg(feature = "javascript")]
mod javascript;

use anyhow::Result;

/// Initialize logging for the BAML runtime
pub fn init_baml_logging() -> Result<()> {
    baml_log::init()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init()
        .ok();
    Ok(())
}

/// Get the current version of the BAML Gleam FFI
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the current log level
pub fn get_log_level() -> &'static str {
    baml_log::get_log_level().as_str()
}

/// Set the log level
pub fn set_log_level(level: &str) -> Result<()> {
    level
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid log level: {}", level))
        .and_then(|lvl| baml_log::set_log_level(lvl).map_err(|e| anyhow::anyhow!("Failed to set log level: {}", e)))
}

/// Enable or disable JSON logging mode
pub fn set_log_json_mode(json: bool) -> Result<()> {
    baml_log::set_json_mode(json).map_err(|e| anyhow::anyhow!("Failed to set JSON mode: {}", e))
}

/// Set the maximum chunk length for log messages
pub fn set_log_max_chunk_length(length: usize) -> Result<()> {
    baml_log::set_max_message_length(length).map_err(|e| anyhow::anyhow!("Failed to set max message length: {}", e))
}

// Export appropriate FFI bindings based on target
// Note: Erlang exports are handled by the rustler::init! macro in erlang.rs

#[cfg(feature = "javascript")]
pub use javascript::*;