//! Delegation mode controlling which update mechanism to use.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Enum {
	/// Use Air if available, otherwise fall through to Tauri.
	#[default]
	AutoDetect,

	/// Use Air exclusively. Errors when the `AirIntegration` feature is off
	/// or the client is unhealthy.
	ForceAir,

	/// Use Tauri's bundled updater exclusively.
	ForceTauri,
}
