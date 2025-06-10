

/**
 * @module error_utils (Handlers)
 * @description This module provides utility functions for creating standardized,
 * serializable error strings for RPC and Tauri command responses. It helps
 * maintain a consistent error reporting format across the application.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod ErrorFormatting;

pub use self::ErrorFormatting::*;
