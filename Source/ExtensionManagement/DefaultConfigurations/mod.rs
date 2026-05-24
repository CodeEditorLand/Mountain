//! `DefaultConfigurations` - atomized.

pub mod CollectDefaultConfigurations;
pub mod ProcessConfigurationProperties;

pub use CollectDefaultConfigurations::Fn as CollectDefaultConfigurations;
pub use ProcessConfigurationProperties::Fn as ProcessConfigurationProperties;
