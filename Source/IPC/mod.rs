//! # IPC Module
//! 
//! Contains the Mountain counterpart to Wind's IPC infrastructure.
//! Provides bidirectional communication between Mountain (Rust backend) and Wind (TypeScript frontend).

#![allow(non_snake_case, non_camel_case_types)]

pub mod TauriIPCServer;
pub mod WindServiceHandlers;
pub mod WindServiceAdapters;
pub mod ConfigurationBridge;
pub mod StatusReporter;
pub mod AdvancedFeatures;
pub mod WindAdvancedSync;

pub use TauriIPCServer::TauriIPCServer;
pub use WindServiceHandlers::register_wind_ipc_handlers;
pub use StatusReporter::initialize_status_reporter;
pub use AdvancedFeatures::initialize_advanced_features;
pub use WindAdvancedSync::initialize_wind_advanced_sync;
