//! # Terminals RPC Service
//!
//! ## ⚠️ Placeholder Module
//!
//! This module is planned for future implementation and will provide:
//! - Terminal emulation services for Cocoon extension host
//! - Pseudo-terminal (PTY) management
//! - Terminal input/output streaming
//! - Shell integration and command execution
//!
//! ## Feature Gate
//!
//! This module is enabled with the `terminals` feature:
//! ```toml
//! [features]
//! terminals = []
//! ```
//!
//! ## Planned API
//!
//! - `TerminalService`: Main service struct for terminal operations
//! - PTY spawning and configuration
//! - Terminal data stream handling
//! - Shell detection and integration
//!
//! TODO: Implement terminal emulation RPC services

/// TerminalService - Stub implementation for terminal emulation RPC services
///
/// This service will handle:
/// - Pseudo-terminal (PTY) spawning
/// - Terminal I/O streaming
/// - Shell integration
#[cfg(feature = "terminals")]
pub struct TerminalService;

#[cfg(feature = "terminals")]
impl TerminalService {
	/// Create a new TerminalService instance
	pub fn new() -> Self { TerminalService }
}

#[cfg(feature = "terminals")]
impl Default for TerminalService {
	fn default() -> Self { Self::new() }
}
