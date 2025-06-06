// File: Handlers/NativeFs/mod.rs
// This module defines and exports handlers for native filesystem operations.
// These are deprecated in favor of the `vscode.workspace.fs` API and its
// corresponding environment provider implementation.

#![allow(non_snake_case, non_camel_case_types)]

mod NativeFs; // Contains the deprecated native filesystem handlers

pub use self::NativeFs::*; // Re-export all public functions from NativeFs.rs
