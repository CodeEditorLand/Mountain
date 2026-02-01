//! # IPC Module - Mountain-Wind Communication Layer
//!
//! **Architectural Role:**
//! This module is Mountain's counterpart to Wind's TypeScript IPC
//! infrastructure. It provides the critical bidirectional communication bridge
//! between Mountain (Rust backend) and Wind (TypeScript frontend), enabling
//! seamless data exchange and command execution across the CodeEditorLand
//! ecosystem.
//!
//! **Communication Patterns:**
//! - **Tauri IPC Event System:** Uses Tauri's event-based IPC for real-time
//!   message passing
//! - **Command-Response Pattern:** Implements request-response semantics for
//!   service operations
//! - **Publish-Subscribe:** Supports event broadcasting to multiple subscribers
//! - **Message Queuing:** Provides offline-scenario support with persistent
//!   message queues
//!
//! **Module Responsibilities:**
//!
//! 1. **TauriIPCServer** - Core IPC server managing connections, message
//!    routing, and security
//!    - Establishes and maintains bidirectional Wind-Mountain communication
//!      channels
//!    - Handles message validation, encryption, and permission checking
//!    - Manages connection pooling for optimal resource utilization
//!    - Implements offline message queuing for reliable delivery
//!
//! 2. **WindServiceHandlers** - Direct command handlers for Wind service
//!    invocations
//!    - Maps Tauri IPC commands to Mountain's internal service calls
//!    - Provides type-safe serialization/deserialization of payloads
//!    - Implements comprehensive error handling and validation
//!    - Handles file system, configuration, storage, and environment operations
//!
//! 3. **WindServiceAdapters** - Type conversion and service bridging layer
//!    - Converts between Wind TypeScript types and Mountain Rust types
//!    - Adapts Wind's service interfaces to Mountain's implementation
//!    - Handles configuration format transformations
//!    - Provides Wind-compatible service abstractions
//!
//! 4. **ConfigurationBridge** - Bidirectional configuration synchronization
//!    - Syncs Mountain backend configuration with Wind frontend
//!    - Implements merge conflict resolution strategies
//!    - Validates configuration changes before application
//!    - Generates unique machine and session IDs for multi-instance support
//!
//! 5. **StatusReporter** - Real-time monitoring and health checking
//!    - Reports IPC status to Sky for centralized monitoring
//!    - Tracks performance metrics (latency, throughput, errors)
//!    - Implements health scoring and automatic recovery mechanisms
//!    - Discovers and monitors all Mountain services
//!
//! 6. **AdvancedFeatures** - Enhanced synchronization capabilities
//!    - Real-time collaboration session management
//!    - Message caching for performance optimization
//!    - Advanced performance monitoring and analytics
//!    - Support for multi-user collaborative editing
//!
//! 7. **WindAdvancedSync** - Real-time document and UI synchronization
//!    - Synchronizes document changes between Wind and Mountain
//!    - Manages UI state across multiple editor windows
//!    - Implements conflict detection and resolution
//!    - Broadcasts real-time updates to subscribers
//!
//! 8. **WindAirCommands** - Air daemon delegation layer
//!    - Allows Wind to delegate background operations to Air daemon
//!    - Implements update management, authentication, and file operations
//!    - Provides search and indexing capabilities via Air
//!    - Handles daemon communication via gRPC
//!
//! **Design Philosophy (Microsoft VSCode-RPC Inspired):**
//! - **Type Safety:** Strong typing between TypeScript (Wind) and Rust
//!   (Mountain)
//! - ** Defensive Validation:** All messages are validated before processing
//! - **Graceful Degradation:** System continues operating in degraded states
//! - **Comprehensive Logging:** All operations logged with appropriate levels
//! - **Performance Monitoring:** Built-in metrics and health checks
//! - **Security First:** Authentication, authorization, and encryption support
//!
//! **Communication Flow:**
//! ```
//! Wind (Frontend)  <--->  TauriIPCServer  <--->  WindServiceHandlers
//!                                            |
//!                                           WindServiceAdapters
//!                                                  |
//!                                  Mountain Internal Services
//!
//! Wind (Frontend)  <--->  ConfigurationBridge  <--->  Settings Service
//!
//! Wind (Frontend)  <--->  WindAirCommands  <--->  Air Daemon (gRPC)
//!
//! StatusReporter  -->  Sky (Monitoring)
//! AdvancedFeatures  -->  Collaboration Sync
//! WindAdvancedSync  -->  Real-time Document Sync
//! ```
//!
//! **Message Serialization:**
//! - All messages use JSON for cross-language compatibility
//! - Binary data base64-encoded for JSON transport
//! - Optional gzip compression for large payloads
//! - Versioned message schemas for backward compatibility
//!
//! **Error Handling Strategy:**
//! 1. Input validation at message entry points
//! 2. Comprehensive error messages with context
//! 3. Automatic retry with exponential backoff for transient failures
//! 4. Graceful degradation when services unavailable
//! 5. Error logging for debugging and monitoring

pub mod TauriIPCServer;
pub mod WindServiceHandlers;
pub mod WindServiceAdapters;
pub mod ConfigurationBridge;
pub mod StatusReporter;
pub mod AdvancedFeatures;
pub mod WindAdvancedSync;
pub mod WindAirCommands;

// Re-export commonly used items for convenience
pub use WindServiceHandlers::register_wind_ipc_handlers;
pub use StatusReporter::initialize_status_reporter;
pub use AdvancedFeatures::initialize_advanced_features;
pub use WindAdvancedSync::initialize_wind_advanced_sync;
pub use WindAirCommands::{
	AirMetricsDTO,
	AirServiceStatusDTO,
	AuthResponseDTO,
	DownloadResultDTO,
	FileResultDTO,
	IndexResultDTO,
	SearchResultsDTO,
	UpdateInfoDTO,
	register_wind_air_commands,
};
