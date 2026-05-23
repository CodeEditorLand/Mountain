//! # StatusBarProvider - Tooltip Resolution
//!
//! Implementation of dynamic tooltip resolution for
//! [`MountainEnvironment`]

use std::sync::Arc;

use CommonLibrary::{Error::CommonError::CommonError, IPC::DTO::ProxyTarget::ProxyTarget};
use serde_json::{Value, json};

use super::super::MountainEnvironment::MountainEnvironment;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::CreateEffectForRequest::Utilities::Proxy::proxy_cocoon,
	dev_log,
};

/// Tooltip resolution operations implementation for MountainEnvironment
pub(super) async fn provide_tooltip_impl(
	env:&MountainEnvironment,

	entry_identifier:String,
) -> Result<Option<Value>, CommonError> {
	dev_log!(
		"lifecycle",
		"[StatusBarProvider] Providing dynamic tooltip for entry: {}",
		entry_identifier
	);

	let run_time:Arc<ApplicationRunTime> = env.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	// This is a "reverse" call, where the host needs data from the sidecar.
	let rpc_response = proxy_cocoon(
		&run_time,
		ProxyTarget::ExtHostStatusBar,
		"ProvideStatusbarTooltip",
		json!([entry_identifier]),
		5000,
	)
	.await
	.map_err(|e| CommonError::IPCError { Description:e })?;

	// If the response is null or fails to parse, we gracefully return None.
	Ok(serde_json::from_value(rpc_response).unwrap_or(None))
}
