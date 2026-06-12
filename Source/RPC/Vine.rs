//! Vine gRPC type re-exports for the RPC layer (placeholder for future
//! cross-service Vine wiring). Two DTOs for now.
/// Vine connection info DTO: carries service name and endpoint for a gRPC
/// connection.
pub mod VineConnectionInfo;

/// Vine service status: indicates Connected, Disconnected, or Error health
/// state.
pub mod VineServiceStatus;
