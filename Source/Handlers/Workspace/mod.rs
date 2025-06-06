// File: Handlers/Workspace/mod.rs
// This module defines and exports handlers for workspace-related operations,
// such as managing workspace folders, trust, and file finding.

#![allow(non_snake_case, non_camel_case_types)]

mod Workspace; // Contains the logic for handling workspace operations

pub use self::Workspace::*; // Re-export all public functions from Workspace.rs
