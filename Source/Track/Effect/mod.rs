//! Effect creation and routing for Track. Two siblings:
//! `CreateEffectForRequest::Fn` (the central router with one domain
//! sub-module per file) and `MappedEffectType::MappedEffect` (the typed
//! result alias). No `pub use` re-exports: callers spell the full path.

/// Central effect router: maps method names to effect constructors.
pub mod CreateEffectForRequest;

/// Typed effect result alias.
pub mod MappedEffectType;
