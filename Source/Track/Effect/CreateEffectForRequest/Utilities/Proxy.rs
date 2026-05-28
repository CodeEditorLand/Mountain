//! Cocoon-proxy helper for `CreateEffectForRequest` handlers that forward
//! calls to the Cocoon Node.js sidecar via `IPCProvider::SendRequestToSideCar`.
//!
//! All 17 proxy handlers follow the same skeleton; `proxy_cocoon` collapses
//! the repeated `Require()` / `format!` / `SendRequestToSideCar` boilerplate
//! into a single await-able call. The caller retains the fallback / dev_log
//! choice since error values differ per handler.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider as IPCProviderTrait},
};
use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn proxy_cocoon(
	run_time:&Arc<ApplicationRunTime>,

	target:ProxyTarget,

	method_suffix:&str,

	params:Value,

	timeout_ms:u64,
) -> Result<Value, String> {
	let ipc:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();

	let method = format!("{}${}", target.GetTargetPrefix(), method_suffix);

	ipc.SendRequestToSideCar("cocoon-main".to_string(), method, params, timeout_ms)
		.await
		.map_err(|e| e.to_string())
}
