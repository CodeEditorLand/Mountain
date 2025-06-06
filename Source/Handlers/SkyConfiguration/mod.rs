// File: Handler/SkyConfiguration/mod.rs
// This module defines and exports the logic for building the initial
// sandbox configuration DTO that is sent to the Sky (frontend) upon startup.

#![allow(non_snake_case, non_camel_case_types)]

mod SkyConfiguration; // Contains the logic for building the SandboxConfigurationDto

pub use self::SkyConfiguration::*; // Re-export all public functions from SkyConfiguration.rs
