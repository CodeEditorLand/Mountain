//! # Binary Module
//!
//! Main module for Binary refactoring, providing clean imports for Binary.rs.
//!
//! This module re-exports all submodules organized by their functionality:
//! - Build: Tauri builder and plugin configuration
//! - Register: Service registration with Tauri
//! - Service: External service initialization
//! - Extension: Extension scanning and population
//! - Shutdown: Graceful shutdown handling
//! - Debug: Debug and trace logging utilities
//! - Initialize: Application initialization
//! - IPC: IPC command modules
//! - Tray: System tray functionality
//! - Main: Main entry point for the application

// Build modules
pub mod Build;

// Register modules
pub mod Register;

// Service modules
pub mod Service;

// Extension modules
pub mod Extension;

// Shutdown modules
pub mod Shutdown;

// Debug module
pub mod Debug;

// Initialize modules
pub mod Initialize;

// IPC command modules
pub mod IPC;

// Tray modules
pub mod Tray;

// Main entry point
pub mod Main;
