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
pub mod WindAirCommands;

// Re-export commonly used items for convenience
pub use WindServiceHandlers::register_wind_ipc_handlers;
pub use StatusReporter::initialize_status_reporter;
pub use AdvancedFeatures::initialize_advanced_features;
pub use WindAdvancedSync::initialize_wind_advanced_sync;
pub use WindAirCommands::{
    register_wind_air_commands,
    UpdateInfoDTO,
    DownloadResultDTO,
    AuthResponseDTO,
    IndexResultDTO,
    SearchResultsDTO,
    FileResultDTO,
    AirServiceStatusDTO,
    AirMetricsDTO,
};
