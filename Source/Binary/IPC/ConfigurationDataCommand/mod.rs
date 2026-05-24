//! `ConfigurationDataCommand` - atomized.

pub mod GetConfigurationData;
pub mod SaveConfigurationData;

pub use GetConfigurationData::Fn as GetConfigurationData;
pub use SaveConfigurationData::Fn as SaveConfigurationData;
