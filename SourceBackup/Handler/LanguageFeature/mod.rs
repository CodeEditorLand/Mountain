// @module language_feature (Handler)
// @description This module contains the core logic for managing and invoking
// all language feature providers. It aggregates and exports the handler
// functions for both provider registration and feature invocation.
//

#![allow(non_snake_case)]

mod InvocationLogic;
mod LanguageFeatureLogic;

pub use self::{InvocationLogic::*, LanguageFeatureLogic::*};
