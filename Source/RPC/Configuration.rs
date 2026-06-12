//! Configuration RPC service. `ConfigurationService::Struct` owns the
//! key/value store; `ConfigurationScope::Enum` and
//! `ConfigurationUpdate::Struct` are the wire DTOs.
pub mod ConfigurationScope;

pub mod ConfigurationService;

pub mod ConfigurationUpdate;
