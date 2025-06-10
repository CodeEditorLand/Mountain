

/**
 * @module language_features (Handlers)
 * @description This module contains the core logic for managing and invoking all
 * language feature providers. It aggregates and exports the handler functions
 * for both provider registration and feature invocation.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod LanguageFeaturesLogic;
mod Support;

pub use self::LanguageFeaturesLogic::*;
pub use self::Support::*;
