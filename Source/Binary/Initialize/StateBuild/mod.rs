//! `StateBuild` - atomized.

pub mod Build;
pub mod BuildWithConfig;
pub mod BuildMinimal;

pub use Build::Fn as Build;
pub use BuildWithConfig::Fn as BuildWithConfig;
pub use BuildMinimal::Fn as BuildMinimal;
