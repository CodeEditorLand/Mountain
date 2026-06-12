//! Cocoon → Mountain submission gate. The `EchoActionServer` wraps every
//! inbound `MountainService` gRPC call in an Echo work-stealing scheduler
//! task tagged with a per-method priority lane (read/write file → High,
//! search/git → Low, default → Normal).
//!
//! Without this gate a `$activateByEvent("*")` fan-out (28+ `ReadFile` +
//! 28+ `Stat` + 28+ `Configuration.Inspect`) starves any interactive Wind
//! IPC arriving during the burst.
pub mod EchoActionServer;

pub mod ExtensionHostRegistry;

pub mod ExtensionRouter;

pub mod ResolveMethodPriority;
