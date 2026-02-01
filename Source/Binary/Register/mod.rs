//! # Register Module
//!
//! Provides service registration functions for Tauri.

pub mod CommandRegister;
pub mod IPCServerRegister;
pub mod StatusReporterRegister;
pub mod AdvancedFeaturesRegister;
pub mod WindSyncRegister;

pub use CommandRegister::CommandRegister;
pub use IPCServerRegister::IPCServerRegister;
pub use StatusReporterRegister::StatusReporterRegister;
pub use AdvancedFeaturesRegister::AdvancedFeaturesRegister;
pub use WindSyncRegister::WindSyncRegister;
