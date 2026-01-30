//! # Mountain Crate Library
//!
//! This file conceptually represents the library root for the Mountain
//! application, declaring all of its major internal components. This allows
//! the `Binary.rs` file to have a clean entry point that orchestrates these
//! components.
//!
//! ## File Responsibilities
//! - Main library entry point for Mountain application
//! - Module declarations for all Mountain components
//! - Mobile entry point configuration for Tauri
//! - Feature flag management and conditional compilation
//! - Cross-platform compatibility definitions
//!
//! ## TODO
//! - [ ] Add comprehensive integration tests for all modules
//! - [ ] Implement proper error handling and recovery patterns
//! - [ ] Add performance monitoring and optimization
//! - [ ] Implement proper logging and diagnostics
//! - [ ] Add security audit and vulnerability assessment
//! - [ ] Implement proper memory management and resource cleanup
//! - [ ] Add comprehensive documentation for all APIs
//! - [ ] Implement proper testing infrastructure
//! - [ ] Add performance benchmarking and profiling
//! - [ ] Implement proper error boundary handling

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
