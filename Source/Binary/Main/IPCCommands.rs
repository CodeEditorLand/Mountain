//! # IPCCommands (Binary/Main)
//!
//! ## RESPONSIBILITIES
//!
//! Tauri command handlers for IPC communication between the frontend and
//! backend. This module delegates to the atomic command modules in Binary/IPC/.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The IPCCommands module is the **command layer** in Mountain's architecture:
//!
//! ```text
//! Frontend ──► Tauri IPC Bridge ──► Binary/IPCC::*Commands ──► Backend Services
//!                                        │
//!                                        ▼
//!                                Command Delegation
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Workbench Configuration**: MountainGetWorkbenchConfiguration
//!   (Binary/IPC/WorkbenchConfigurationCommand)
//! - **IPC Messaging**: MountainIPCReceiveMessage, MountainIPCGetStatus,
//!   MountainIPCInvoke (Binary/IPC/*)
//! - **Wind Desktop**: MountainGetWindDesktopConfiguration,
//!   MountainUpdateConfigurationFromWind (Binary/IPC/WindConfigurationCommand)
//! - **Configuration**: MountainSynchronizeConfiguration,
//!   MountainGetConfigurationStatus (Binary/IPC/ConfigurationSyncCommand)
//! - **Configuration Data**: MountainGetConfigurationData,
//!   MountainSaveConfigurationData (Binary/IPC/ConfigurationDataCommand)
//! - **IPC Status**: MountainGetIPCStatus, MountainGetIPCStatusHistory,
//!   MountainStartIPCStatusReporting (Binary/IPC/IPCStatusCommand)
//! - **Performance**: MountainGetPerformanceStats, MountainGetCacheStats
//!   (Binary/IPC/PerformanceStatsCommand)
//! - **Collaboration**: MountainCreateCollaborationSession,
//!   MountainGetCollaborationSessions (Binary/IPC/CollaborationSessionCommand)
//! - **Document Sync**: MountainAddDocumentForSync, MountainGetSyncStatus,
//!   MountainSubscribeToUpdates (Binary/IPC/DocumentSyncCommand)
//!
//! ## ERROR HANDLING
//!
//! All commands return `Result<serde_json::Value, String>` for consistency.
//! Errors are logged and converted to user-friendly error messages.
//! Uses `map_err` and `and_then` for ergonomic error handling.
//!
//! ## LOGGING
//!
//! Each command logs at INFO level on request receipt, DEBUG for processing
//! details, and ERROR for failures. All logs are prefixed with `[IPC]
//! [CommandName]`.
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! All commands are async with `tokio` runtime support.
//! Minimal allocation overhead by reusing existing data structures.
//!
//! ## TODO
//! - [ ] Add request rate limiting
//! - [ ] Implement command validation middleware
//! - [ ] Add telemetry for command usage analytics

// Note: All Tauri IPC commands are now in the atomic command modules
// in Binary/IPC/*.rs. This module is kept for organizational purposes.
//
// Command modules:
// - WorkbenchConfigurationCommand
// - MessageReceiveCommand
// - StatusGetCommand
// - InvokeCommand
// - WindConfigurationCommand
// - ConfigurationUpdateCommand
// - ConfigurationSyncCommand
// - ConfigurationStatusCommand
// - ConfigurationDataCommand
// - IPCStatusCommand
// - IPCStatusHistoryCommand
// - IPCStatusReportingStartCommand
// - PerformanceStatsCommand
// - CollaborationSessionCommand
// - DocumentSyncCommand
// - UpdateSubscriptionCommand
// - CacheStatsCommand
