// File: Environment/mod.rs
// This file serves as the module declaration for the MountainEnvironment and
// its providers.

// Sub-modules for different environment provider implementations.
// Each of these will contain the actual logic for the traits implemented by
// MountainEnvironment.
pub(crate) mod CommandsProvider;
pub(crate) mod ConfigProvider;
pub(crate) mod DiagnosticsProvider;
pub(crate) mod DocumentsProvider;
pub(crate) mod FilesystemProvider; // Renamed from FsProvider
pub(crate) mod IpcProvider;
pub(crate) mod LanguageFeaturesProvider;
pub(crate) mod OutputProvider;
pub(crate) mod SecretsProvider;
pub(crate) mod StorageProvider;
pub(crate) mod UiProvider;
pub(crate) mod WorkspaceProvider;

// Internal utilities specific to the Environment module.
pub(crate) mod Utils;

// Re-exporting the primary MountainEnvironment struct from its own file.
mod MountainEnvironment;
pub use self::MountainEnvironment::MountainEnvironment;
