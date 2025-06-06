// File: AppState/HierarchySessionContext.rs
// Defines the context for an ongoing call hierarchy or type hierarchy session,
// linking subsequent requests (like getting incoming/outgoing calls) back to
// the original provider that initiated the session.

#![allow(non_snake_case, non_camel_case_types)]

use Common::LanguageFeatureEffect::ProviderType as CommonLanguageProviderType;

#[derive(Debug, Clone)]
pub struct HierarchySessionContext {
	// The handle of the provider that started this hierarchy session.
	pub OriginalProviderHandle:u32,
	// The identifier of the sidecar where the provider is running.
	pub OriginalSidecarIdentifier:String,
	// The type of hierarchy session (e.g., CallHierarchy, TypeHierarchy).
	pub ProviderType:CommonLanguageProviderType,
}
