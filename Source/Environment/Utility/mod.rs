//! Cross-cutting utilities shared by every `Environment` provider: error
//! mapping, language detection, workspace-trust path validation, URI
//! parsing, and context-key when-clause evaluation. Callers spell the
//! full path (`Environment::Utility::ErrorMapping::Fn`, etc.) - no
//! `pub use` re-exports.

pub mod EnhanceShellEnvironment;

pub mod GlobPattern;

pub mod TextEdit;

pub mod ErrorMapping;

pub mod LanguageDetection;

pub mod PathSecurity;

pub mod UriParsing;

pub mod WhenClause;
