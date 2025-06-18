// @module DiagnosticProvider (Environment)
// @description Implements the `DiagnosticsProvider` trait for
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	diagnostic::DiagnosticsProvider,
	Environment::Requires,
	error::CommonError,
	language_feature::DTO::MarkerDataDTO, // Diagnostics uses the Marker DTO
};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::diagnostic as DiagnosticHandler;

#[async_trait]
impl DiagnosticsProvider for MountainEnvironment {
	// Sets or updates diagnostics for multiple resources from a specific
	// owner.
	async fn SetDiagnostics(&self, owner:String, entries_DTO_value:Value) -> Result<(), CommonError> {
		DiagnosticHandler::SetDiagnosticsLogic(&self.ApplicationHandle, owner, entries_DTO_value).await
	}

	// Clears all diagnostics from a specific owner.
	async fn ClearDiagnostics(&self, owner:String) -> Result<(), CommonError> {
		DiagnosticHandler::ClearDiagnosticsLogic(&self.ApplicationHandle, owner).await
	}

	// Retrieves all diagnostics, optionally filtered by a resource URI.
	async fn GetAllDiagnostics(&self, resource_uri_filter:Option<Value>) -> Result<Value, CommonError> {
		DiagnosticHandler::GetAllDiagnosticsLogic(&self.ApplicationHandle, resource_uri_filter).await
	}
}

impl Requires<Arc<dyn DiagnosticsProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DiagnosticsProvider + Send + Sync> { Arc::new(self.clone()) }
}
