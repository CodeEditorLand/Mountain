// @module error_utils (Handler)
// @description This module provides utility functions for creating
// standardized, serializable error strings for RPC and Tauri command responses.
// It helps maintain a consistent error reporting format across the application.
// Renamed from ErrorUtility for consistency.

#![allow(non_snake_case)]

mod ErrorFormatting;

pub use self::ErrorFormatting::*;
