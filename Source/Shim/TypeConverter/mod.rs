// File: Shim/TypeConverter/mod.rs
// This module file is conceptual for Rust, as the `cocoon-type-converters`
// directory exists in the TypeScript codebase. In a parallel Rust structure,
// this would declare and re-export converter modules.

#![allow(non_snake_case, non_camel_case_types)]

// In a Rust implementation, this `mod.rs` would look something like this:
mod CodeAction;
mod Completion;
mod LanguageFeature;
mod Main;
mod Notebook; // Added for completeness
mod WorkspaceEdit;

pub use self::{CodeAction::*, Completion::*, LanguageFeature::*, Main::*, Notebook::*, WorkspaceEdit::*};
