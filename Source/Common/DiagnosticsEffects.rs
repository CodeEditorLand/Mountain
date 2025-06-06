// File: Common/DiagnosticsEffects.rs
// Defines the DiagnosticsManager trait and associated effects for managing
// diagnostic markers (errors, warnings, etc.) within the application.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

// DTO for a diagnostic marker (re-exported for convenience if needed).
// This assumes MarkerDataDto is defined elsewhere, like in LanguageFeatureEffects.
pub use crate::LanguageFeatureEffects::MarkerDataDto;
use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can manage diagnostic collections.
#[async_trait]
pub trait DiagnosticsManager: Environment {
	/// Sets or clears diagnostics for a given owner and set of resources.
	async fn SetDiagnostics(&self, Owner:String, EntriesDtoValue:Value) -> Result<(), CommonError>;
	/// Clears all diagnostics for a given owner.
	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError>;
	/// Retrieves all diagnostics, optionally filtered by a specific resource.
	async fn GetAllDiagnostics(&self, ResourceUriFilterOption:Option<Value>) -> Result<Value, CommonError>;
}

/// Creates an effect to set diagnostics for a specific owner.
pub fn SetDiagnostics<RuntimeAccessType>(
	Owner:String,
	EntriesDtoValue:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DiagnosticsManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OwnerClone = Owner.clone();
		let EntriesClone = EntriesDtoValue.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn DiagnosticsManager> = Environment.require();
			Manager.SetDiagnostics(OwnerClone, EntriesClone).await
		})
	}))
}

/// Creates an effect to clear all diagnostics for a specific owner.
pub fn ClearDiagnostics<RuntimeAccessType>(Owner:String) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DiagnosticsManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OwnerClone = Owner.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn DiagnosticsManager> = Environment.require();
			Manager.ClearDiagnostics(OwnerClone).await
		})
	}))
}

/// Creates an effect to get all diagnostics, with an optional resource filter.
pub fn GetAllDiagnostics<RuntimeAccessType>(
	ResourceUriFilterOption:Option<Value>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Value>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DiagnosticsManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let FilterClone = ResourceUriFilterOption.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn DiagnosticsManager> = Environment.require();
			Manager.GetAllDiagnostics(FilterClone).await
		})
	}))
}
