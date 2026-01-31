//! # Library
//!
//! Root library module for the Mountain application, declaring all major
//! internal components and providing a clean organization structure.
//!
//! ## RESPONSIBILITIES
//!
//! ### Module Organization
//! - Declare and export all Mountain subsystem modules
//! - Provide clear module boundaries and visibility rules
//! - Enable clean dependency management between components
//! - Support both library and binary builds
//!
//! ### Module Categories
//!
//! **Core Infrastructure:**
//! - `ApplicationState`: Centralized application state management
//! - `Environment`: Capability provider and dependency injection
//! - `RunTime`: Effect execution engine (ApplicationRunTime)
//!
//! **Communication & Integration:**
//! - `IPC`: Inter-process communication with Wind frontend
//! - `Air`: AI/integration service client
//! - `Vine`: gRPC inter-service communication server
//!
//! **Service Management:**
//! - `ProcessManagement`: Cocoon sidecar process lifecycle
//! - `FileSystem`: File system operations and monitoring
//! - `ExtensionManagement`: Extension discovery and management
//!
//! **Commands & Features:**
//! - `Command`: Native command implementations
//! - `Track`: Command tracking and dispatch
//! - `WorkSpace`: Workspace file parsing and management
//!
//! **Entry Point:**
//! - `Binary`: Main application entry point (binary crate)
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Library root for the Mountain crate
//! - Central module declaration point
//! - Provides module visibility control
//!
//! ### Dependencies
//! - Common: Shared infrastructure (Environment, ApplicationRunTime trait)
//! - Echo: Task scheduling (via RunTime)
//! - Tauri: Desktop framework integration
//!
//! ### Dependents
//! - Binary: Uses all modules for application initialization
//! - Frontend (Sky): Interacts via IPC and Commands
//!
//! ## TODO
//!
//! ### Immediate Improvements
//! - Add module-level documentation references
//! - Implement feature flags for optional subsystems
//! - Add public module organization chart
//!
//! ### Future Work
//! - Consider separating into multiple crates (core, services, commands)
//! - Add module dependency graph documentation
//! - Implement module versioning for stability guarantees
//! - Add automatic module loading for plugins
//!
//! ### Missing Functionality to Probe
//! - Optimal module granularity for compilation performance
//! - Module re-export strategy for external consumers
//! - Cross-compilation considerations for mobile targets

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types)]
#![feature(trivial_bounds)]

pub mod Air;

pub mod ApplicationState;

pub mod Command;

pub mod Environment;

pub mod ExtensionManagement;

pub mod FileSystem;

pub mod IPC;

pub mod ProcessManagement;

pub mod RunTime;

pub mod Track;

pub mod Vine;

pub mod WorkSpace;

// The main binary entry point is defined in its own module.
pub mod Binary;

/// The main entry point for mobile builds, which is required by Tauri but
/// delegates to the primary binary logic.
#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() { Binary::Fn(); }
