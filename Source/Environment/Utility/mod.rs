#![allow(non_snake_case)]

//! Cross-cutting utilities shared by every `Environment` provider: error
//! mapping, language detection, workspace-trust path validation, and URI
//! parsing. Callers spell the full path
//! (`Environment::Utility::ErrorMapping::Fn`, etc.) - no `pub use`
//! re-exports.

pub mod EnhanceShellEnvironment;

pub mod ErrorMapping;

pub mod LanguageDetection;

pub mod PathSecurity;

pub mod UriParsing;
