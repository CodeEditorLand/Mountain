// ---------------------------------------------------------------------------------------------
// Mountain Environment - Diagnostics Provider
// 
// --------------------------------------------------------------------------------------------
// This module implements the `DiagnosticsManager` trait for
// `MountainEnvironment`. It manages diagnostic collections (errors, warnings,
// etc.) from various sources (e.g., linters, language servers) and associates
// them with resource URIs. Operations are delegated to handler functions in
// `handlers::diagnostics`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	diagnostics_effects::DiagnosticsManager, // The trait being implemented
	environment::Requires,
	errors::CommonError,
	// MarkerDataDto is implicitly handled as part of `Value` in the trait,
	// but handlers will need to work with the concrete DTO from `Land_Common`.
};
use async_trait::async_trait;
use log::{info, trace}; // For logging
use serde_json::Value; // For flexible DTO structures

use crate::{
	environment::MountainEnvironment,
	handlers, // For delegating to diagnostics handlers
};

// --- DiagnosticsManager Implementation ---
#[async_trait]
impl DiagnosticsManager for MountainEnvironment {
	async fn set_diagnostics(
		&self,
		owner:String,
		entries_dto_val:Value, // Expected: Array of [UriComponentsValue, Option<Vec<MarkerDataDtoAsValue>>]
	) -> Result<(), CommonError> {
		info!(
			"[Env DiagProv] SetDiagnostics: Owner='{}', NumEntries={}",
			owner,
			entries_dto_val.as_array().map_or(0, |a| a.len())
		);
		trace!("[Env DiagProv] SetDiagnostics Full DTO: {:?}", entries_dto_val);

		// Delegate to the handler function.
		// The handler in `handlers::diagnostics` will parse `entries_dto_val`,
		// convert `MarkerDataDto` (from common) to `app_state::MarkerData` (internal),
		// update `AppState`, and emit Tauri events.
		handlers::diagnostics::handle_set_diagnostics_effect_logic(self.app_handle.clone(), owner, entries_dto_val)
			.await
	}

	async fn clear_diagnostics(&self, owner:String) -> Result<(), CommonError> {
		info!("[Env DiagProv] ClearDiagnostics: Owner='{}'", owner);

		// Delegate to the handler function.
		handlers::diagnostics::handle_clear_diagnostics_effect_logic(self.app_handle.clone(), owner).await
	}

	async fn get_all_diagnostics(
		&self,
		resource_uri_filter_opt:Option<Value>, // Optional UriComponentsValue to filter by
	) -> Result<Value, CommonError> {
		trace!("[Env DiagProv] GetAllDiagnostics: filter='{:?}'", resource_uri_filter_opt);

		// Delegate to the handler function.
		// The handler will aggregate `app_state::MarkerData`, convert to
		// `common::MarkerDataDto`, and construct the response Value.
		handlers::diagnostics::handle_get_all_diagnostics_effect_logic(self.app_handle.clone(), resource_uri_filter_opt)
			.await
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn DiagnosticsManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DiagnosticsManager + Send + Sync> { Arc::new(self.clone()) }
}
