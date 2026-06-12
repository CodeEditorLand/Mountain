//! Extension-host → Mountain submission gate. The `EchoActionServer` wraps
//! every inbound `MountainService` gRPC call in an Echo work-stealing scheduler
//! task tagged with a per-method priority lane (read/write file → High,
//! search/git → Low, default → Normal).
//!
//! Without this gate a `$activateByEvent("*")` fan-out (28+ `ReadFile` +
//! 28+ `Stat` + 28+ `Configuration.Inspect`) starves any interactive Wind
//! IPC arriving during the burst.
/// Echo action server: singleton submission gate that dispatches extension host
/// requests.
pub mod EchoActionServer;

/// Extension host registry: maps extension identifiers to host identifiers.
pub mod ExtensionHostRegistry;

/// Extension router: resolves the owning host for a given extension identifier.
pub mod ExtensionRouter;

/// Resolve method priority: maps gRPC wire method names to Echo scheduler
/// priority lanes.
pub mod ResolveMethodPriority;
