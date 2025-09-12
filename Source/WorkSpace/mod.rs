// File: Mountain/Source/WorkSpace/mod.rs
// Role: Public module interface for workspace-related logic.
// Responsibilities:
//   - Expose services related to managing workspaces, such as parsing
//     `.code-workspace` files.

//! # WorkSpace Module
//!
//! This module is responsible for managing workspaces, including parsing
//! `.code-workspace` files and handling workspace-level state.

#![allow(non_snake_case, non_camel_case_types)]

pub mod WorkSpaceFileService;
