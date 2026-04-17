//! # Windows RPC Service
//!
//! ## ⚠️ Placeholder Module
//!
//! This module is planned for future implementation and will provide:
//! - Window management services for Groove and Cocoon extension hosts
//! - Document window creation and lifecycle management
//! - Webview panel hosting and communication
//!
//! ## Feature Gate
//!
//! This module is enabled with the `grove` or `cocoon` features:
//! ```toml
//! [features]
//! grove = []
//! cocoon = []
//! ```
//!
//! ## Planned API
//!
//! - `WindowService`: Main service struct for window operations
//! - Window creation and destruction handlers
//! - Window state management and serialization
//!
//! TODO: Implement window management RPC services

/// WindowService - Stub implementation for window management RPC services
///
/// This service will handle:
/// - Window creation and destruction
/// - Document window lifecycle management
/// - Webview panel hosting
#[cfg(any(feature = "grove", feature = "cocoon"))]
pub struct WindowService;

#[cfg(any(feature = "grove", feature = "cocoon"))]
impl WindowService {
	/// Create a new WindowService instance
	pub fn new() -> Self { WindowService }
}

#[cfg(any(feature = "grove", feature = "cocoon"))]
impl Default for WindowService {
	fn default() -> Self { Self::new() }
}
