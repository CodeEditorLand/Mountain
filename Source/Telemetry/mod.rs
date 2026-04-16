//! # Telemetry and Observability System
//!
//! This module provides comprehensive telemetry, tracing, metrics, and feature
//! flag capabilities for the Mountain application. It integrates OpenTelemetry
//! (OTEL) standards for distributed tracing and metrics collection.
//!
//! ## Module Structure
//!
//! - **Gates**: Compile-time and runtime feature gates for
//!   Debug/Development/Telemetry builds
//! - **Tracing**: OpenTelemetry distributed tracing integration
//! - **Metrics**: Performance and operational metrics collection
//! - **FeatureFlags**: Runtime feature flag management
//!
//! ## Build Configuration
//!
//! ### Feature Flags
//!
//! Add these to `Cargo.toml` features section:
//!
//! ```toml
//! [features]
//! default = ["ExtensionHostCocoon", "MistNative", "AirIntegration"]
//!
//! # Build-time telemetry configuration
//! Telemetry = ["tracing", "opentelemetry"]
//! Development = ["Telemetry", "devtools"]
//! Debug = ["Development"]
//!
//! # Runtime feature flags
//! RuntimeFeatureFlags = []
//! MetricsCollection = ["Telemetry"]
//! DistributedTracing = ["Telemetry"]
//! ```
//!
//! ### Usage Examples
//!
//! ```text
//! use Mountain::Telemetry::*;
//! #[cfg(feature = "Telemetry")]
//! use tracing::{info, instrument};
//!
//! #[cfg(feature = "Telemetry")]
//! #[instrument(skip(env))]
//! async fn process_command(env:&MountainEnvironment, command:Command) {
//! 	dev_log!("lifecycle", "Processing command: {:?}", command);
//! 	// ... command processing logic
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Zero-cost abstractions**: Disabled in release builds when Telemetry
//!   feature is off
//! - **Async-friendly**: All telemetry operations are async-aware
//! - **Low overhead**: Sampling and filtering to minimize performance impact
//! - **Thread-safe**: All metrics can be collected from multiple threads
//!
//! ## OTEL Integration
//!
//! When the `Telemetry` feature is enabled, the system:
//! - Creates spans for all RPC calls
//! - Collects metrics for IPC operations
//! - Tracks command execution times
//! - Monitors IPC connection health
//! - Records extension lifecycle events

// ============================================================================
// Core Telemetry Modules
// ============================================================================

/// Build-time and runtime feature gates
pub mod Gates;

/// OpenTelemetry distributed tracing
#[cfg(feature = "Telemetry")]
pub mod Tracing;

/// Performance and operational metrics
#[cfg(feature = "Telemetry")]
pub mod Metrics;

/// Runtime feature flag management
pub mod FeatureFlags;

// ============================================================================
// Public API
// ============================================================================

/// Initialize telemetry system
#[cfg(feature = "Telemetry")]
pub fn initialize_telemetry() -> Result<(), Box<dyn std::error::Error>> {
	use Tracing::{initialize_metrics, initialize_tracing};

	initialize_tracing()?;
	initialize_metrics()?;

	Ok(())
}

/// Check if telemetry is enabled
#[inline]
pub fn is_telemetry_enabled() -> bool { cfg!(feature = "Telemetry") }

/// Check if this is a debug build
#[inline]
pub fn is_debug_build() -> bool { cfg!(debug_assertions) }

/// Check if this is a development build
#[inline]
pub fn is_development_build() -> bool { cfg!(feature = "Development") || cfg!(debug_assertions) }
use crate::dev_log;
