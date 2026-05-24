//! # StatusBarProvider - Tooltip Resolution
//!
//! Implementation of dynamic tooltip resolution for
//! [`MountainEnvironment`]

use std::sync::Arc;

use CommonLibrary::{Error::CommonError::CommonError, IPC::DTO::ProxyTarget::ProxyTarget};
use tauri::Manager;
use serde_json::{Value, json};

use super::super::MountainEnvironment::Struct;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::CreateEffectForRequest::Utilities::Proxy::proxy_cocoon,
	dev_log,
};

/// Tooltip resolution operations implementation for MountainEnvironment
pub(super) async fn provide_tooltip_impl(
	env:&MountainEnvironment,

	EntryIdentifier:String,
) -> Result<Option<Value>, CommonError> {
	dev_log!(
		"lifecycle",
		"[StatusBarProvider] Providing dynamic tooltip for entry: {}",
		EntryIdentifier
	);

	let RunTime:Arc<ApplicationRunTime> = env.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	// This is a "reverse" call, where the host needs data from the sidecar.
	let RpcResponse = ProxyCocoon(
		&RunTime,
		ProxyTarget::ExtHostStatusBar,
		"ProvideStatusbarTooltip",
		json!([EntryIdentifier]),
		5000,
	)
	.await
	.map_err(|E| CommonError::IPCError { Description:e })?;

	// If the response is null or fails to parse, we gracefully return None.
	Ok(serde_json::from_value(RpcResponse).unwrap_or(None))
}
