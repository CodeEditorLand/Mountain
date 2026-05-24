//! `InitializationData` - atomized.

pub mod ConstructSandboxConfiguration;
pub mod ConstructExtensionHostInitializationData;

pub use ConstructSandboxConfiguration::Fn as ConstructSandboxConfiguration;
pub use ConstructExtensionHostInitializationData::Fn as ConstructExtensionHostInitializationData;
