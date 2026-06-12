//! # Vine gRPC Server
//!
//! Implements the Mountain gRPC server for incoming connections from sidecars
//! like Cocoon. Handles RPC request dispatching into Mountain application
//! logic.
//!
//! ## Architecture
//!
//! Two complementary gRPC services:
//!
//! - **MountainService**: Handles Cocoon-to-Mountain requests and notifications
//!   (listening port)
//! - **CocoonService**: Sends Mountain-to-Cocoon requests and notifications
//!   (outgoing port)
//!
//! ## Lifecycle
//!
//! 1. **Initialization**: Servers spawned as background tasks via `Initialize`
//! 2. **Service Registration**: gRPC services registered with tonic's Server
//!    builder
//! 3. **Request Handling**: Each RPC call dispatched to appropriate handlers
//! 4. **Graceful Shutdown**: Servers terminate when tokio runtime shuts down
//!
//! ## Security
//!
//! - Request size limits (4 MB default)
//! - Method whitelisting
//! - Parameter validation before processing
//! - Safe error messages (no sensitive data leakage)
//!
//! ## Sub-modules
//!
//! - [`Initialize`]: Server initialization and startup
//! - [`MountainVinegRPCService`]: MountainService impl (Cocoon → Mountain)
//! - [`VineHostImpl`]: Vine host implementation
//! - [`Notification`]: Cocoon-to-Mountain notification atoms (one handler per
//!   file)

/// Server initialization and startup.
pub mod Initialize;

/// MountainService implementation (handles Cocoon → Mountain calls).
pub mod MountainVinegRPCService;

/// Vine host implementation.
pub mod VineHostImpl;

/// Cocoon-to-Mountain notification atoms. One handler per file so the
/// dispatcher stays a thin match; each wire method lives at
/// `Vine::Server::Notification::<Atom>::<Atom>` for grep-friendly
/// navigation.
pub mod Notification;
