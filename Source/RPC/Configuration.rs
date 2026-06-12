//! Configuration RPC service. `ConfigurationService::Struct` owns the
//! key/value store; `ConfigurationScope::Enum` and
//! `ConfigurationUpdate::Struct` are the wire DTOs.
pub mod ConfigurationScope;

/// Configuration service: reads and writes key/value settings on behalf of the
/// extension host.
pub mod ConfigurationService;

/// Configuration update DTO: carries a key, value, and scope for a single
/// configuration change.
pub mod ConfigurationUpdate;
