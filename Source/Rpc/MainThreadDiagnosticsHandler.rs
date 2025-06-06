// File: Rpc/MainThreadDiagnosticsHandler.rs
// Defines the RPC handler for diagnostics-related requests from the sidecar.
// This includes setting, clearing, and retrieving diagnostic markers.

use std::sync::Arc;

use Common::{DiagnosticsEffects, Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::Runtime::AppRuntime; // Mountain's AppRuntime
use crate::{
	Handlers::ErrorUtils,
	Rpc::Args::Diagnostics::{ChangeManyArgument as ChangeManyDiagnosticsArgument, GetDiagnosticsArgument},
};

#[derive(Clone)]
pub struct MainThreadDiagnosticsHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadDiagnosticsHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Sets or clears diagnostics for multiple resources from a specific owner.
	pub async fn ChangeMany(&self, Argument:ChangeManyDiagnosticsArgument) -> Result<Value, String> {
		info!(
			"[Rpc DiagnosticsHandler] ChangeMany (DTO): Owner='{}', EntriesDtoIsArray={}",
			Argument.Owner,
			Argument.EntriesDtoValue.is_array()
		);
		trace!(
			"[Rpc DiagnosticsHandler] ChangeMany Entries DTO: {:?}",
			Argument.EntriesDtoValue
		);

		let Effect = DiagnosticsEffects::SetDiagnostics(Argument.Owner.clone(), Argument.EntriesDtoValue);
		self.Runtime.Run(Effect).await.map(|_| Value::Null).map_err(|Error| {
			ErrorUtils::MapCommonErrorToRpcString(Error, &format!("SetDiagnostics for Owner '{}'", Argument.Owner))
		})
	}

	/// Clears all diagnostics for a specific owner.
	/// This method was part of the original `diagnostics.rs` but not directly
	/// in `track.rs` dispatch. It's included here for completeness if it's
	/// intended to be an RPC endpoint. If not, it might be an internal
	/// AppState operation or an effect not directly exposed via this handler.
	pub async fn Clear(&self, Owner:String) -> Result<Value, String> {
		info!(
			"[Rpc DiagnosticsHandler] Clear (DTO flow, assuming DTO from direct call): Owner='{}'",
			Owner
		);
		let Effect = DiagnosticsEffects::ClearDiagnostics(Owner.clone());
		self.Runtime.Run(Effect).await.map(|_| Value::Null).map_err(|Error| {
			ErrorUtils::MapCommonErrorToRpcString(Error, &format!("ClearDiagnostics for Owner '{}'", Owner))
		})
	}

	/// Retrieves all diagnostics, optionally filtered by a resource URI.
	pub async fn GetDiagnostics(&self, Argument:GetDiagnosticsArgument) -> Result<Value, String> {
		trace!(
			"[Rpc DiagnosticsHandler] GetDiagnostics (DTO): Filter='{:?}'",
			Argument.ResourceUriFilterOption
		);

		let Effect = DiagnosticsEffects::GetAllDiagnostics(Argument.ResourceUriFilterOption);
		self.Runtime
			.Run(Effect)
			.await
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "GetAllDiagnostics DTO"))
	}
}
