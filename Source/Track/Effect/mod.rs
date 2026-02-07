//! # Effect Module (Track)
//!
//! Contains the effect creation and routing functionality for the Track module.

mod CreateEffectForRequest;
mod MappedEffectType;

// Re-export with both original name and Fn alias for backward compatibility
pub use CreateEffectForRequest::CreateEffectForRequest;
pub use CreateEffectForRequest::CreateEffectForRequest as Fn;
pub use MappedEffectType::MappedEffect;
