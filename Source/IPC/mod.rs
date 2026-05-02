//! # IPC Module
//!
//! ## RESPONSIBILITIES

#![allow(unused_imports, unused_variables)]
//! Inter-process communication (IPC) for the Mountain application, handling
//! communication between the Tauri frontend and the Rust backend through
//! various protocols including Tauri commands, WebSocket, and custom message
//! formats.
//!
//! ### Core Functions:
//! - **Message Routing**: Route IPC messages to appropriate handlers
//! - **Connection Management**: Manage IPC connections with health monitoring
//! - **Security**: Implement permission system for IPC access control
//! - **Encryption**: Provide secure message channels and compression
//! - **Status Reporting**: Report IPC system status and metrics
//! - **Configuration Bridge**: Bridge configuration across IPC boundaries
//! - **Wind Sync**: Advanced synchronization with Wind UI framework
//! - **Advanced Features**: Experimental/advanced IPC features
//!
//! ## Architectural Role
//!
//! The IPC module is the **communication layer** in Mountain's architecture:
//!
//! ```text
//! Sky (Frontend) ──► IPC (Communication) ──► Track (Dispatch) ──► Services
//! Wind (UI) ───────────────────────────────────────────────────────────────┘
//! Cocoon (Sidecar) ──► Vine (gRPC) ────────────────────────────┘
//! ```
//!
//! ### Design Principles:
//! 1. **Protocol Agnostic**: Support multiple IPC protocols
//! 2. **Security First**: All communications are secured and permission-gated
//! 3. **High Performance**: Optimized for low-latency communication
//! 4. **Observable**: Comprehensive logging and metrics
//!
//! ## Key Components
//!
//! - **TauriIPCServer**: Main IPC server orchestrator
//! - **Message**: Message types and routing
//! - **Connection**: Connection management and health
//! - **Encryption**: Message compression and secure channels
//! - **Security**: Permission system
//! - **ConfigurationBridge**: Configuration synchronization
//! - **StatusReporter**: Status and metrics reporting
//! - **WindAdvancedSync**: Wind framework integration
//! - **AdvancedFeatures**: Advanced/experimental features
//!
//! ## TODOs
//! High Priority:
//! - [ ] Add comprehensive unit tests for all modules
//! - [ ] Implement connection pooling optimizations
//! - [ ] Add connection timeout handling
//!
//! Medium Priority:
//! - [ ] Add message batching for efficiency
//! - [ ] Implement keep-alive packets
//! - [ ] Add connection retry logic
//!
//! Low Priority:
//! - [ ] Add message persistence for offline mode
//! - [ ] Implement message compression ratio optimization
//! - [ ] Add connection encryption rotation

// --- Main Sub-modules ---

/// Common shared types and abstractions for IPC layer.
pub mod Common;

/// Main Tauri IPC server orchestrator.
// Legacy TauriIPCServer.rs for backward compatibility
#[path = "TauriIPCServer.rs"]
pub mod TauriIPCServer_Old;

/// Message types and routing.
pub mod Message;

/// Connection management and health monitoring.
pub mod Connection;

/// Message compression and secure channels.
pub mod Encryption;

/// Permission system for IPC access control.
pub mod Security;

// --- Feature Sub-modules ---

/// Advanced experimental features (collaboration, intelligent caching,
/// performance monitoring). TODO: atomize this 648-LOC single file into a
/// directory; for now consumers spell `IPC::AdvancedFeatures::*` directly.
pub mod AdvancedFeatures;

/// Configuration synchronization bridge.
// Legacy ConfigurationBridge.rs for backward compatibility
#[path = "ConfigurationBridge.rs"]
pub mod ConfigurationBridge;

/// Status and metrics reporting.
// Legacy StatusReporter.rs for backward compatibility
#[path = "StatusReporter.rs"]
pub mod StatusReporter;

/// Wind UI framework synchronization.
// Legacy WindAdvancedSync.rs for backward compatibility
#[path = "WindAdvancedSync.rs"]
pub mod WindAdvancedSync;

// --- Legacy Sub-modules ---

/// Legacy Wind Air Commands.
pub mod WindAirCommands;

/// Legacy Wind Service Adapters.
pub mod WindServiceAdapters;

/// Tag-filtered development logging (Trace env var).
/// Must be declared before WindServiceHandlers so the dev_log! macro is
/// available.
pub mod DevLog;

/// Central `sky://` emit wrapper that logs under the `sky-emit` DevLog
/// tag. Optional drop-in for any `ApplicationHandle::emit(channel, …)`
/// call site; existing emits keep working unchanged.
pub mod SkyEmit;

/// Shared `UriComponents` emitter. Every handler that returns a URI to the
/// renderer must route through this module so the `$mid: 1` marshalling
/// marker is never forgotten (without it VS Code's IPC reviver skips the
/// field and `uri.with is not a function` cascades through the sidebar).
pub mod UriComponents;

/// Wind Service Handlers - dispatcher for every `MountainIPCInvoke` Tauri
/// call from Wind/Output/Sky. The `mod.rs` inside is the central `match`
/// that routes wire strings to per-domain atoms or handler files. Atoms
/// live under `WindServiceHandlers/<Domain>/<Atom>.rs` following the
/// one-export-per-file convention.
///
/// The previous `WindServiceHandler` (singular) sibling was merged here
/// on 2026-04-23: of its 24 files, only 3 functions were live
/// (extensions install/uninstall, nativeHost showOpenDialog) and those
/// now live as atoms under `WindServiceHandlers/Extension/` and
/// `WindServiceHandlers/NativeDialog/`. The remaining 21 files were
/// dead-code duplicates of plural-side implementations.
pub mod WindServiceHandlers;

// --- Legacy Subdirectories ---

/// Legacy Enhanced subdirectory.
pub mod Enhanced;

/// Legacy Permission subdirectory.
pub mod Permission;

// No `pub use` re-exports - callers spell the full path
// (`IPC::Connection::Manager::ConnectionManager`, etc.). The legacy single-
// file modules `TauriIPCServer_Old`, `AdvancedFeatures`, `StatusReporter`,
// `WindAdvancedSync`, `ConfigurationBridge` remain as roots for the
// in-progress atomic migration.

// --- Notes on Migration ---

// MIGRATION PATH TO ATOMIC STRUCTURE:
//
// Phase 1: ✅ Create Atomic Structure
// - Created new atomic module directories
// - Implemented core functionality
// - Added comprehensive documentation
//
// Phase 2: 🔄 Backward Compatibility (Current)
// - Keeping legacy files for compatibility
// - Using #[path = "..."] to reference legacy files
// - Gradually migrating dependent code
//
// Phase 3: ⏳ Migration
// - Update dependent files to use new structure
// - Test migration incrementally
// - Monitor for issues
//
// Phase 4: ⏳ Cleanup
// - Remove legacy files
// - Update all documentation
// - Final verification
//
// The following atomic modules are ready for migration:
// - IPC/TauriIPCServer/ (Server.rs)
// - IPC/Message/ (Types.rs)
// - IPC/Connection/ (Manager.rs, Types.rs, Health.rs)
// - IPC/Encryption/ (MessageCompressor.rs, SecureChannel.rs)
// - IPC/Security/ (PermissionManager.rs, Role.rs, Permission.rs)
// - IPC/AdvancedFeatures/ (Features.rs)
// - IPC/ConfigurationBridge/ (mod.rs - placeholder)
// - IPC/StatusReporter/ (mod.rs - placeholder)
// - IPC/WindAdvancedSync/ (mod.rs - placeholder)
