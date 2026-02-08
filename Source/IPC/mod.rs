//! # IPC Module
//!
//! ## RESPONSIBILITIES
//!
//! Inter-process communication (IPC) for the Mountain application, handling
//! communication between the Tauri frontend and the Rust backend through various
//! protocols including Tauri commands, WebSocket, and custom message formats.
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
// Note: Legacy TauriIPCServer.rs is used for backward compatibility
// TODO: Migrate to TauriIPCServer/mod.rs in future phase
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

/// Advanced experimental features.
// Note: Legacy AdvancedFeatures.rs is used for backward compatibility
// TODO: Migrate to AdvancedFeatures/mod.rs in future phase
#[path = "AdvancedFeatures.rs"]
pub mod AdvancedFeatures;

/// Configuration synchronization bridge.
// Note: Legacy ConfigurationBridge.rs is used for backward compatibility
// TODO: Migrate to ConfigurationBridge/mod.rs in future phase
#[path = "ConfigurationBridge.rs"]
pub mod ConfigurationBridge;

/// Status and metrics reporting.
// Note: Legacy StatusReporter.rs is used for backward compatibility
// TODO: Migrate to StatusReporter/mod.rs in future phase
#[path = "StatusReporter.rs"]
pub mod StatusReporter;

/// Wind UI framework synchronization.
// Note: Legacy WindAdvancedSync.rs is used for backward compatibility
// TODO: Migrate to WindAdvancedSync/mod.rs in future phase
#[path = "WindAdvancedSync.rs"]
pub mod WindAdvancedSync;

// --- Legacy Sub-modules ---

/// Legacy Wind Air Commands.
pub mod WindAirCommands;

/// Legacy Wind Service Adapters.
pub mod WindServiceAdapters;

/// Legacy Wind Service Handlers.
pub mod WindServiceHandlers;

// --- Legacy Subdirectories ---

/// Legacy Enhanced subdirectory.
pub mod Enhanced;

/// Legacy Permission subdirectory.
pub mod Permission;

// --- Re-exports for backward compatibility ---

pub use Common::{ConnectionStatus, HealthStatus, MessageType, PerformanceMetrics, ServiceInfo};
pub use Message::SimpleConnectionStatus;
pub use TauriIPCServer_Old as TauriIPCServer;
pub use Message::{TauriIPCMessage, ListenerCallback};
pub use Connection::{ConnectionHandle, ConnectionManager, ConnectionStats, HealthChecker};
pub use Encryption::MessageCompressor::MessageCompressor;
pub use Encryption::SecureChannel::{SecureMessageChannel, EncryptedMessage};
pub use Security::PermissionManager::{PermissionManager, SecurityContext, SecurityEvent, SecurityEventType};
pub use Security::Role::Role;
pub use AdvancedFeatures::{AdvancedFeatures as AdvancedFeatures_New, initialize_advanced_features, CollaborationSession, CollaborationPermissions, PerformanceStats, MessageCache, CachedMessage};
pub use ConfigurationBridge as ConfigurationBridge_New;
pub use StatusReporter as StatusReporter_New;
pub use WindAdvancedSync as WindAdvancedSync_New;

// --- Legacy compatibility function re-exports ---

// Note: initialize_advanced_features is already exported above

pub use StatusReporter::initialize_status_reporter;
pub use WindAdvancedSync::initialize_wind_advanced_sync;

// --- Notes on Migration ---

/*
MIGRATION PATH TO ATOMIC STRUCTURE:

Phase 1: ✅ Create Atomic Structure
- Created new atomic module directories
- Implemented core functionality
- Added comprehensive documentation

Phase 2: 🔄 Backward Compatibility (Current)
- Keeping legacy files for compatibility
- Using #[path = "..."] to reference legacy files
- Gradually migrating dependent code

Phase 3: ⏳ Migration
- Update dependent files to use new structure
- Test migration incrementally
- Monitor for issues

Phase 4: ⏳ Cleanup
- Remove legacy files
- Update all documentation
- Final verification

The following atomic modules are ready for migration:
- IPC/TauriIPCServer/ (Server.rs)
- IPC/Message/ (Types.rs)
- IPC/Connection/ (Manager.rs, Types.rs, Health.rs)
- IPC/Encryption/ (MessageCompressor.rs, SecureChannel.rs)
- IPC/Security/ (PermissionManager.rs, Role.rs, Permission.rs)
- IPC/AdvancedFeatures/ (Features.rs)
- IPC/ConfigurationBridge/ (mod.rs - placeholder)
- IPC/StatusReporter/ (mod.rs - placeholder)
- IPC/WindAdvancedSync/ (mod.rs - placeholder)
*/
