//! Extension-domain handlers for `CocoonService`.
//! `GetAllExtensions::Fn`, `GetConfiguration::Fn`, `GetExtension::Fn`.
/// GetAllExtensions handler: retrieves all installed extensions.
pub mod GetAllExtensions;

/// GetConfiguration handler: retrieves extension-specific configuration.
pub mod GetConfiguration;

/// GetExtension handler: retrieves a single extension by identifier.
pub mod GetExtension;
