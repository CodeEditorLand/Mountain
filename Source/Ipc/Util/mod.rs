// File: Ipc/Util/mod.rs
// Declares and exports utility modules for the IPC system.

#![allow(non_snake_case, non_camel_case_types)]

// This module contains helper functions for converting between
// `serde_json::Value` and `google.protobuf.Value`.
mod ProtoValueConverter;

// Re-export all public items from the converter module.
pub use self::ProtoValueConverter::*;
