#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// ============================================================================
// Element: Mountain - 50-Level Deep Analysis
// ============================================================================
//
// Overview
// --------
// Mountain is the main Tauri-based desktop application in the Land monorepo.
//
// Level 1-10: Basic Structure
// ---------------------------
// | Level | Task                        | Status |
// |-------|-----------------------------|--------|
// | 1     | Verify Cargo.toml exists    | ✅     |
// | 2     | Check Source/ directory structure | ✅ |
// | 3     | Identify main modules       | ✅     |
// | 4     | Check for build.rs          | ✅     |
// | 5     | Verify .gitattributes (LFS) | ✅     |
// | 6     | Check .github/workflows     | ✅     |
// | 7     | Identify Dependencies       | ✅     |
// | 8     | Check Binary/ directory     | ✅     |
// | 9     | Verify documentation        | ✅     |
// | 10    | Check Target/ directory     | ⬜     |
//
// Level 11-20: Module Analysis
// ----------------------------
// | Level | Task                          | Status |
// |-------|-------------------------------|--------|
// | 11    | Analyze ApplicationState module | ⬜   |
// | 12    | Analyze Binary module         | ⬜     |
// | 13    | Analyze Command module        | ⬜     |
// | 14    | Analyze Environment module    | ⬜     |
// | 15    | Analyze Error module          | ⬜     |
// | 16    | Analyze ExtensionManagement module | ⬜ |
// | 17    | Analyze FileSystem module     | ⬜     |
// | 18    | Analyze IPC module            | ⬜     |
// | 19    | Analyze ProcessManagement module | ⬜  |
// | 20    | Analyze RPC module            | ⬜     |
//
// Level 21-30: Code Quality Checks (TODOs: 265)
// ----------------------------------------------
// | Level | Task                        | Status         |
// |-------|-----------------------------|----------------|
// | 21    | Check for unused imports    | ✅ From f2cc266 |
// | 22    | Check for dead code         | ⬜             |
// | 23    | Check TODO comments (265!)  | 🔴 Priority    |
// | 24    | Verify naming conventions   | ⬜             |
// | 25    | Check error handling in RPC | ⬜             |
// | 26    | Verify logging patterns     | ⬜             |
// | 27    | Check for magic numbers     | ⬜             |
// | 28    | Verify async patterns       | ⬜             |
// | 29    | Check thread safety         | ✅ From 78ebab1, f2cc266 |
// | 30    | Verify test coverage        | ⬜             |
//
// Level 31-40: Convention Verification
// -------------------------------------
// | Level | Task                       | Status    |
// |-------|----------------------------|-----------|
// | 31    | Verify PascalCase          | ✅ Verified |
// | 32    | Verify snake_case files    | ⬜         |
// | 33    | Check module organization  | ✅        |
// | 34    | Verify struct naming       | ⬜         |
// | 35    | Verify enum naming         | ⬜         |
// | 36    | Check function naming      | ⬜         |
// | 37    | Verify trait naming        | ⬜         |
// | 38    | Check constant naming      | ⬜         |
// | 39    | Verify type aliases        | ⬜         |
// | 40    | Check visibility modifiers | ⬜         |
//
// Level 41-50: Refactoring Priorities
// ------------------------------------
// | Level | Task                         | Status       |
// |-------|------------------------------|--------------|
// | 41    | Address RPC/CocoonService TODOs | 🔴 20+ TODOs |
// | 42    | Check for code duplication   | ⬜           |
// | 43    | Verify DRY principles        | ⬜           |
// | 44    | Check SOLID principles       | ⬜           |
// | 45    | Identify performance issues  | ⬜           |
// | 46    | Check security considerations| ⬜           |
// | 47    | Verify error messages        | ⬜           |
// | 48    | Check documentation completeness | ⬜       |
// | 49    | Final Mountain-specific audit | ⬜          |
// | 50    | Complete Mountain analysis   | ⬜           |
//
// TODO Breakdown for Mountain
// ----------------------------
// - **RPC/CocoonService.rs**: ~30+ TODOs (provider registrations)
// - **Other files**: ~235+ TODOs
// - **Priority**: High - language server features
//
// Summary for Mountain
// --------------------
// - **Type**: Rust/Tauri Desktop Application
// - **TODOs**: 265 found 🔴
// - **Key Changes**: Thread safety (RwLock), naming consistency, LFS
// - **Last Commit Changes**: NC-09, NC-10, NC-11, TS-01, TS-02, ARCH-31, ARCH-32
//
// Last Updated: 2026-03-03
// ============================================================================

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
//! - Workspace: Workspace file parsing
//!
//! ### Entry Point
//! - Binary: Main application entry point

// ============================================================================
// Core Infrastructure Modules
// ============================================================================

/// Centralized error handling system
pub mod Error;

pub mod ApplicationState;

pub mod Environment;

pub mod RunTime;

// ============================================================================
// Communication Modules
// ============================================================================

pub mod IPC;

pub mod Air;

pub mod Vine;

pub mod RPC;

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

pub mod Workspace;

// ============================================================================
// Entry Point Module
// ============================================================================

pub mod Binary;

// ============================================================================
// Mobile Build Entry Point
// ============================================================================

/// Main entry point for both mobile and desktop builds.
/// - On mobile: marked as Tauri mobile entry point
/// - On desktop: serves as the standard binary entry point
/// Delegates to the primary binary logic in the Binary module.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() { Binary::Main::Main(); }
