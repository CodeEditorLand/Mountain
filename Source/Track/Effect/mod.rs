//! Effect creation and routing for `Track`. Two siblings:
//! `CreateEffectForRequest::Fn` (the central router with one domain
//! sub-module per file) and `MappedEffectType::MappedEffect` (the typed
//! result alias). No `pub use` re-exports - callers spell the full path.

pub mod CreateEffectForRequest;

pub mod MappedEffectType;
