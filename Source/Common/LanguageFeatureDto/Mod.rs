// File: Common/LanguageFeatureDto/Mod.rs
// This module defines and exports all Data Transfer Objects (DTOs) related to
// language features, providing a structured way to communicate complex data
// for operations like completion, hover, symbols, etc.

#![allow(non_snake_case, non_camel_case_types)]

// Sub-modules for specific DTOs
mod OptionsDto;

// Re-export all DTOs for easy access from other modules.
pub use self::OptionsDto::SpecificProviderOptionsDto;
// The primary, large DTO file is assumed to be named after the module.
mod LanguageFeatureDto;
pub use self::LanguageFeatureDto::*;
