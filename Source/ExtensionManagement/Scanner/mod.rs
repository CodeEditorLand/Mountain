//! `Scanner` - atomized.

pub mod ScanDirectoryForExtensions;
pub mod CollectDefaultConfigurations;

pub use ScanDirectoryForExtensions::Fn as ScanDirectoryForExtensions;
pub use CollectDefaultConfigurations::Fn as CollectDefaultConfigurations;
