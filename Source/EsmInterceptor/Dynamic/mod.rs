// File: EsmInterceptor/Dynamic/mod.rs
// Declares and exports modules related to the dynamic generation of the
// 'vscode' module script for ES Module imports.

#![allow(non_snake_case, non_camel_case_types)]

// This module contains the function that creates the dynamic script content.
mod Dynamic;
// This module contains the template string for the dynamic script.
mod DynamicTemplate;

// Re-export the primary script creation function.
pub use self::Dynamic::CreateDynamicVscodeModuleScript;
