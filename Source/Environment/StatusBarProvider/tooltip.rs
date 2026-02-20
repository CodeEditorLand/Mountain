//! # StatusBarProvider - Tooltip Resolution
//!
//! Implementation of dynamic tooltip resolution for
//! [`MountainEnvironment`]

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use log::info;
use serde_json::{Value, json};

use super::super::MountainEnvironment::MountainEnvironment;

/// Tooltip resolution operations implementation for MountainEnvironment
pub(super) async fn provide_tooltip_impl(
	env:&MountainEnvironment,
	entry_identifier:String,
) -> Result<Option<Value>, CommonError> {
	info!("[StatusBarProvider] Providing dynamic tooltip for entry: {}", entry_identifier);

	let ipc_provider:Arc<dyn IPCProvider> = env.Require();

	// This is a "reverse" call, where the host needs data from the sidecar.
	let rpc_method = format!("{}$ProvideStatusbarTooltip", ProxyTarget::ExtHostStatusBar.GetTargetPrefix());

	let rpc_response = ipc_provider
		.SendRequestToSideCar("cocoon-main".to_string(), rpc_method, json!([entry_identifier]), 5000)
		.await?;

	// If the response is null or fails to parse, we gracefully return None.
	Ok(serde_json::from_value(rpc_response).unwrap_or(None))
}
