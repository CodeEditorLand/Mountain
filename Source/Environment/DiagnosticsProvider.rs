// File: Environment/DiagnosticsProvider.rs
// Implements the `DiagnosticsManager` trait for the `MountainEnvironment`.
// This file connects abstract diagnostic effects to the concrete logic
// in the application's diagnostics handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{DiagnosticsEffect::DiagnosticsManager, Environment::Requires, Errors::CommonError};
use async_trait::async_trait;
use log::{info, trace};
use serde_json::Value;

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl DiagnosticsManager for MountainEnvironment {
	/// Sets or clears diagnostics for a given owner.
	async fn SetDiagnostics(&self, Owner:String, EntriesDtoValue:Value) -> Result<(), CommonError> {
		info!(
			"[Environment DiagnosticsProvider] SetDiagnostics: Owner='{}', EntryCount={}",
			Owner,
			EntriesDtoValue.as_array().map_or(0, |a| a.len())
		);
		trace!(
			"[Environment DiagnosticsProvider] SetDiagnostics Full DTO: {:?}",
			EntriesDtoValue
		);

		// Delegate the core logic to the specific handler function.
		Handlers::Diagnostics::HandleSetDiagnosticsEffectLogic(self.AppHandle.clone(), Owner, EntriesDtoValue).await
	}

	/// Clears all diagnostics for a given owner.
	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError> {
		info!("[Environment DiagnosticsProvider] ClearDiagnostics: Owner='{}'", Owner);

		Handlers::Diagnostics::HandleClearDiagnosticsEffectLogic(self.AppHandle.clone(), Owner).await
	}

	/// Retrieves all diagnostics, with an optional filter for a specific
	/// resource URI.
	async fn GetAllDiagnostics(&self, ResourceUriFilterOption:Option<Value>) -> Result<Value, CommonError> {
		trace!(
			"[Environment DiagnosticsProvider] GetAllDiagnostics: Filter='{:?}'",
			ResourceUriFilterOption
		);

		Handlers::Diagnostics::HandleGetAllDiagnosticsEffectLogic(self.AppHandle.clone(), ResourceUriFilterOption).await
	}
}

impl Requires<Arc<dyn DiagnosticsManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DiagnosticsManager + Send + Sync> { Arc::new(self.clone()) }
}
