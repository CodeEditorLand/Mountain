//! # IPC
//!
//! IPC command handlers for the Mountain binary.
//!
//! ## RESPONSIBILITIES
//!
//! ### Module Organization
//! - Export all IPC command modules
//! - Provide unified interface for Tauri command registration
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Top-level IPC module in Binary subsystem
//! - Bridge between Tauri and internal services
//!
//! ### Dependencies
//! - tauri: IPC framework
//! - All IPC command submodules
//!
//! ### Dependents
//! - Binary: Registers commands with Tauri invoke handler
//! - Tauri invoke handler: Routes incoming IPC calls
//!
//! ## SECURITY
//!
//! ### Considerations
//! - All commands follow Tauri security model
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Async commands don't block main thread

pub mod WorkbenchConfigurationCommand;
pub mod TrayIconSwitchCommand;
pub mod MessageReceiveCommand;
pub mod StatusGetCommand;
pub mod InvokeCommand;
pub mod WindConfigurationCommand;
pub mod ConfigurationUpdateCommand;
pub mod ConfigurationSyncCommand;
pub mod ConfigurationStatusCommand;
pub mod ConfigurationDataCommand;
pub mod IPCStatusCommand;
pub mod IPCStatusHistoryCommand;
pub mod IPCStatusReportingStartCommand;
pub mod PerformanceStatsCommand;
pub mod CollaborationSessionCommand;
pub mod DocumentSyncCommand;
pub mod UpdateSubscriptionCommand;
pub mod CacheStatsCommand;

// Re-export all commands for convenient registration
pub use WorkbenchConfigurationCommand::MountainGetWorkbenchConfiguration;
pub use TrayIconSwitchCommand::SwitchTrayIcon;
pub use MessageReceiveCommand::MountainIPCReceiveMessage;
pub use StatusGetCommand::MountainIPCGetStatus;
pub use InvokeCommand::MountainIPCInvoke;
pub use WindConfigurationCommand::MountainGetWindDesktopConfiguration;
pub use ConfigurationUpdateCommand::MountainUpdateConfigurationFromWind;
pub use ConfigurationSyncCommand::MountainSynchronizeConfiguration;
pub use ConfigurationStatusCommand::MountainGetConfigurationStatus;
pub use ConfigurationDataCommand::{GetConfigurationData, SaveConfigurationData};
pub use IPCStatusCommand::MountainGetIPCStatus;
pub use IPCStatusHistoryCommand::MountainGetIPCStatusHistory;
pub use IPCStatusReportingStartCommand::MountainStartIPCStatusReporting;
pub use PerformanceStatsCommand::MountainGetPerformanceStats;
pub use CacheStatsCommand::MountainGetCacheStats;
pub use CollaborationSessionCommand::{MountainCreateCollaborationSession, MountainGetCollaborationSessions};
pub use DocumentSyncCommand::{MountainAddDocumentForSync, MountainGetSyncStatus};
pub use UpdateSubscriptionCommand::MountainSubscribeToUpdates;
