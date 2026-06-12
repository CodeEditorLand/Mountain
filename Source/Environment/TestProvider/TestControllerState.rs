//! Per-controller registration record. Carries the extension-provided
//! identifier, label, owning sidecar, active flag, and supported test
//! type tags. Stored in `TestProviderState::Struct` keyed by
//! `ControllerIdentifier`.

use serde::{Deserialize, Serialize};

/// Registration record for a single test controller.
///
/// Stores the extension-provided identifier, user-facing label, owning
/// sidecar process, active flag, and a list of supported test type tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	/// Unique identifier for this controller (e.g. `"rust-analyzer"`).
	pub ControllerIdentifier:String,
	/// User-facing label shown in the Test Explorer panel.
	pub Label:String,
	/// Sidecar process that owns this controller, if any.
	pub SideCarIdentifier:Option<String>,
	/// Whether this controller is currently active.
	pub IsActive:bool,
	/// Test type tags this controller supports (e.g. `"unit"`, `"integration"`).
	pub SupportedTestTypes:Vec<String>,
}
