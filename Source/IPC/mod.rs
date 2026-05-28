//! Inter-process communication for Mountain: Tauri commands, Wind sync,
//! Air daemon client, and Sky event emission.

pub mod Common;

#[path = "TauriIPCServer.rs"]
pub mod TauriIPCServer_Old;

pub mod AdvancedFeatures;

#[path = "ConfigurationBridge.rs"]
pub mod ConfigurationBridge;

pub mod StatusReporter;

#[path = "WindAdvancedSync.rs"]
pub mod WindAdvancedSync;

pub mod WindServiceAdapters;

pub mod DevLog;

pub mod SkyEmit;

pub mod EmitWithTraceparent;

pub mod UriComponents;

pub mod WindServiceHandlers;
