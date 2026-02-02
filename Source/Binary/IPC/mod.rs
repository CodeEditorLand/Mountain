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
