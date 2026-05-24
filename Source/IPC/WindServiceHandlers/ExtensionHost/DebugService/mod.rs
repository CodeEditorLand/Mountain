//! `DebugService` - atomized.

pub mod ExtensionHostDebugReload;
pub mod ExtensionHostDebugClose;

pub use ExtensionHostDebugReload::Fn as ExtensionHostDebugReload;
pub use ExtensionHostDebugClose::Fn as ExtensionHostDebugClose;
