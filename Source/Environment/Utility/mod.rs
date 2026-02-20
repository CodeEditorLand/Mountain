//! # Environment Utilities
//!
//! Shared utility functions used across all Environment provider
//! implementations in
//! [`MountainEnvironment`](crate::Environment::MountainEnvironment::MountainEnvironment).
//! These handle cross-cutting concerns: error mapping, security validation,
//! language detection, and URI conversions.
//!
//! This module acts as a facade, re-exporting all public items from its
//! submodules to maintain backward compatibility.
//!
//! ## Module Organization
//! - [`ErrorMapping`]: Error conversion utilities
//! - [`LanguageDetection`]: Language identifier detection
//! - [`PathSecurity`]: Workspace trust and path validation
//! - [`UriParsing`]: URI/URL parsing and conversion

// Submodules
pub mod ErrorMapping;
pub mod LanguageDetection;
pub mod PathSecurity;
pub mod UriParsing;

// Re-exports for backward compatibility
pub use ErrorMapping::*;
pub use LanguageDetection::*;
pub use PathSecurity::*;
pub use UriParsing::*;
