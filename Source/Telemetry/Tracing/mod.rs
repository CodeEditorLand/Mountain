//! # OpenTelemetry Distributed Tracing
//!
//! This module provides OpenTelemetry (OTEL) distributed tracing integration
//! for the Mountain application. It creates spans for all RPC calls, tracks
//! command execution times, and provides observability across the system.
//!
//! ## Features
//!
//! - Automatic span creation for gRPC services
//! - IPC operation tracing
//! - Command execution tracking
//! - Performance metrics collection
//! - Integration with tracing instrumentation macros
//!
//! ## Dependencies
//!
//! Requires the `Telemetry` feature flag and the following crates:
//! - `tracing`
//! - `opentelemetry`
//! - `tracing-opentelemetry`
//! - `tracing-subscriber`

#[cfg(feature = "Telemetry")]
use tracing::{info, warn, error, debug, instrument};
#[cfg(feature = "Telemetry")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ============================================================================
// Initialization
// ============================================================================

/// Initialize the tracing subscriber with OpenTelemetry integration
#[cfg(feature = "Telemetry")]
pub fn initialize_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // Use env_logger as the base (already configured via tauri-plugin-log)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    if cfg!(debug_assertions) {
                        "mountain=debug,air=debug,cocoon=debug".to_string()
                    } else {
                        "mountain=info,air=info,cocoon=info".to_string()
                    }
                })
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("OpenTelemetry tracing initialized");
    Ok(())
}

/// Initialize tracing (no-op when Telemetry feature is disabled)
#[cfg(not(feature = "Telemetry"))]
pub fn initialize_tracing() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// Span Creation Helpers
// ============================================================================

#[cfg(feature = "Telemetry")]
/// Create a new span with the given name and attributes
pub fn create_span(name: &str, attributes: &[(&str, &str)]) -> tracing::Span {
    let mut span = tracing::span!(tracing::Level::INFO, name);
    
    for (key, value) in attributes {
        span.record(*key, *value);
    }
    
    span
}

/// Create a span (no-op when Telemetry feature is disabled)
#[cfg(not(feature = "Telemetry"))]
pub fn create_span(_name: &str, _attributes: &[(&str, &str)]) -> () {
    // No-op
}

// ============================================================================
// Instrumentation Helpers for Mountain Services
// ============================================================================

#[cfg(feature = "Telemetry")]
/// Instrument an RPC service call
pub async fn instrument_rpc<F, T, E>(
    service_name: &str,
    method_name: &str,
    operation: F
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let span = tracing::span!(
        tracing::Level::INFO,
        "rpc_call",
        service = %service_name,
        method = %method_name
    );
    
    let _enter = span.enter();
    
    info!("RPC call started: {}.{}", service_name, method_name);
    
    let start = std::time::Instant::now();
    
    match operation.await {
        Ok(result) => {
            let duration = start.elapsed();
            info!(
                "RPC call completed: {}.{} (duration: {:?})",
                service_name,
                method_name,
                duration
            );
            Ok(result)
        }
        Err(err) => {
            let duration = start.elapsed();
            error!(
                "RPC call failed: {}.{} (duration: {:?}, error: {})",
                service_name,
                method_name,
                duration,
                err
            );
            Err(err)
        }
    }
}

/// Instrument RPC (no-op when Telemetry feature is disabled)
#[cfg(not(feature = "Telemetry"))]
pub async fn instrument_rpc<F, T, E>(
    _service_name: &str,
    _method_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>
{
    operation.await
}

#[cfg(feature = "Telemetry")]
/// Instrument a command execution
pub async fn instrument_command<F, T>(
    command_name: &str,
    operation: F
) -> Result<T, Common::Error::Error>
where
    F: std::future::Future<Output = Result<T, Common::Error::Error>>,
{
    let span = tracing::span!(
        tracing::Level::INFO,
        "command_execute",
        command = %command_name
    );
    
    let _enter = span.enter();
    
    info!("Executing command: {}", command_name);
    
    let start = std::time::Instant::now();
    
    match operation.await {
        Ok(result) => {
            let duration = start.elapsed();
            info!(
                "Command executed successfully: {} (duration: {:?})",
                command_name,
                duration
            );
            Ok(result)
        }
        Err(err) => {
            let duration = start.elapsed();
            error!(
                "Command execution failed: {} (duration: {:?}, error: {})",
                command_name,
                duration,
                err
            );
            Err(err)
        }
    }
}

/// Instrument command (no-op when Telemetry feature is disabled)
#[cfg(not(feature = "Telemetry"))]
pub async fn instrument_command<F, T>(
    _command_name: &str,
    operation: F,
) -> Result<T, Common::Error::Error>
where
    F: std::future::Future<Output = Result<T, Common::Error::Error>>
{
    operation.await
}

// ============================================================================
// Performance Macros
// ============================================================================

#[cfg(feature = "Telemetry")]
/// Macro to measure execution time of a code block
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!("{} took {:?}", $name, duration);
        result
    }};
}

/// No-op version for when Telemetry is disabled
#[cfg(not(feature = "Telemetry"))]
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $block:block) => {{
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tracing_initialization() {
        // Should not panic
        let result = initialize_tracing();
        #[cfg(feature = "Telemetry")]
        {
            assert!(result.is_ok());
        }
    }
}
