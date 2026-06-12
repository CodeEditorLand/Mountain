//! Cross-cutting utilities shared by every `Environment` provider: error
//! mapping, language detection, workspace-trust path validation, URI
//! parsing, and context-key when-clause evaluation. Callers spell the
//! full path (`Environment::Utility::ErrorMapping::Fn`, etc.) - no
//! `pub use` re-exports.

/// Appends environment variables for extension contributions before PTY spawns.
pub mod EnhanceShellEnvironment;

/// Matches file paths against glob patterns with brace expansion support.
pub mod GlobPattern;

/// Applies text edits (insert, replace, delete) to a string buffer.
pub mod TextEdit;

/// Converts Common/Internal errors to `CommonError` for provider return types.
pub mod ErrorMapping;

/// Detects language identifiers from file extensions and shebang lines.
pub mod LanguageDetection;

/// Validates file paths against workspace-trust security policies.
pub mod PathSecurity;

/// Parses and constructs file, folder, and web URIs for provider interfaces.
pub mod UriParsing;

/// Evaluates `when` clause context-key expressions for command enablement.
pub mod WhenClause;
