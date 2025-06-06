// File: Handler/Secrets/mod.rs
// This module defines and exports handlers for secure secret storage,
// interacting with the operating system's keyring or equivalent credential
// manager.

#![allow(non_snake_case, non_camel_case_types)]

mod Secrets; // Contains the logic for getting, setting, and deleting secrets

pub use self::Secrets::*; // Re-export all public functions from Secrets.rs
