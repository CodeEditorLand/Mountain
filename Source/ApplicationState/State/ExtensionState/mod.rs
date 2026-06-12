//! Extension-related state. Three sub-stores (`ExtensionRegistry`,
//! `ProviderRegistration`, `ScannedExtensions`) plus the composite `State`
//! struct. Callers spell the full sub-path.

/// Extensionregistry module.
pub mod ExtensionRegistry;

/// Providerregistration module.
pub mod ProviderRegistration;

/// Scannedextensions module.
pub mod ScannedExtensions;

/// State module.
pub mod State;
