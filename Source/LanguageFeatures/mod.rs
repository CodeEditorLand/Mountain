
// Declares and exports the primary logic handler for language features.

#![allow(non_snake_case, non_camel_case_types)]

// This module contains the main logic for registering providers and handling
// feature requests.
mod LanguageFeatures;

// Re-export all public items from the main logic file.
pub use self::LanguageFeatures::*;
