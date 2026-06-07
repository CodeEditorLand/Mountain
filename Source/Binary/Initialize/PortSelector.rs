//! # PortSelector
//!
//! Selects an unused port for the localhost server.
//!
//! ## RESPONSIBILITIES
//!
//! ### Port Selection
//! - Find an unused port on localhost
//! - Return port number for server binding
//! - Handle port selection errors
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Early initialization component in Binary subsystem
//! - Provides port for localhost server
//!
//! ### Dependencies
//! - portpicker: Port selection utility
//!
//! ### Dependents
//! - Fn() main entry point: Uses selected port
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Ensure port is not previously bound by other services
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Port selection should be fast
//! - Consider port caching for development

/// Select an unused port for the localhost server.
///
/// Finds an available port on localhost and returns it.
///
/// # Returns
///
/// Returns the selected port number.
///
/// # Panics
///
/// Panics if port selection fails.
pub fn Select() -> u16 {
	portpicker::pick_unused_port().expect("FATAL: Failed to find a free port for Localhost Server")
}

/// Build localhost URL from port number.
///
/// Creates a localhost URL from the provided port number.
///
/// # Arguments
///
/// * `Port` - Port number
///
/// # Returns
///
/// Returns the localhost URL string.
pub fn BuildUrl(Port:u16) -> String { format!("http://localhost:{}", Port) }
