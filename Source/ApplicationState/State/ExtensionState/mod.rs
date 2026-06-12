//! Extension-related state. Three sub-stores (`ExtensionRegistry`,
//! `ProviderRegistration`, `ScannedExtensions`) plus the composite `State`
//! struct. Callers spell the full sub-path.

/// Extension command-handle registry.
pub mod ExtensionRegistry;

/// Language-provider registration state.
pub mod ProviderRegistration;

/// Discovered-extension manifest state.
pub mod ScannedExtensions;

/// Composite extension state container.
pub mod State;
