#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

//! # Library
//!
//! Library root module for the Mountain application, declaring all subsystem
//! modules and providing the main entry point for the Tauri desktop framework.
//!
//! ## RESPONSIBILITIES
//!
//! ### Module Organization
//! - Declare and export all Mountain subsystem modules
//! - Provide clean module boundaries and visibility rules
//! - Enable dependency management between components
//! - Support both library and binary builds
//!
//! ### Entry Point
//! - Provide mobile build entry point required by Tauri
//! - Delegate to Binary module for main application logic
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Library root for the Mountain crate
//! - Central module declaration point
//! - Provides module visibility control
//!
//! ### Dependencies
//! - Common: Shared infrastructure
//! - Tauri: Desktop framework integration
//!
//! ### Dependents
//! - Binary: Uses all modules for application initialization
//!
//! ## MODULE STRUCTURE
//!
//! ### Core Infrastructure
//! - ApplicationState: Centralized state management
//! - Environment: Capability provider and dependency injection
//! - RunTime: Effect execution engine
//!
//! ### Communication
//! - IPC: Inter-process communication
//! - Air: AI/integration service client
//! - Vine: gRPC inter-service communication
//!
//! ### Service Management
//! - ProcessManagement: Sidecar process lifecycle
//! - FileSystem: File system operations
//! - ExtensionManagement: Extension discovery
//!
//! ### Commands & Features
//! - Command: Native command implementations
//! - Track: Command tracking and dispatch
//! - WorkSpace: Workspace file parsing
//!
//! ### Entry Point
//! - Binary: Main application entry point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// ============================================================================
// Core Infrastructure Modules
// ============================================================================

pub mod ApplicationState;

pub mod Environment;

pub mod RunTime;

// ============================================================================
// Communication Modules
// ============================================================================

pub mod IPC;

pub mod Air;

pub mod Vine;

// ============================================================================
// Service Management Modules
// ============================================================================

pub mod ProcessManagement;

pub mod FileSystem;

pub mod ExtensionManagement;

// ============================================================================
// Command And Feature Modules
// ============================================================================

pub mod Command;

pub mod Track;

pub mod WorkSpace;

// ============================================================================
// Entry Point Module
// ============================================================================

pub mod Binary;

// ============================================================================
// Mobile Build Entry Point
// ============================================================================

/// Main entry point for mobile builds, which is required by Tauri but
/// delegates to the primary binary logic in the Binary module.
#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn Main() { Binary::Main::Fn(); }
