// File: Handlers/ExtensionStatus/mod.rs
// This module defines and exports handlers for processing extension lifecycle
// status notifications from the sidecar, such as activation events and errors.

mod ExtensionStatus; // Contains the main logic for handling extension status notifications

pub use self::ExtensionStatus::*; // Re-export all public functions from ExtensionStatus.rs
