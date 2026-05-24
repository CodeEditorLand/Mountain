//! Cocoon-proxy helper for `CreateEffectForRequest` handlers that forward
//! calls to the Cocoon Node.js sidecar via `IPCProvider::SendRequestToSideCar`.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider as IPCProviderTrait},
};
use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(
	RunTime:&Arc<ApplicationRunTime>,
	Target:ProxyTarget,
	MethodSuffix:&str,
	Params:Value,
	TimeoutMs:u64,
) -> Result<Value, String> {
	let Ipc:Arc<dyn IPCProviderTrait> = RunTime.Environment.Require();
	let Method = format!("{}${}", Target.GetTargetPrefix(), MethodSuffix);
	Ipc.SendRequestToSideCar("cocoon-main".to_string(), Method, Params, TimeoutMs)
		.await
		.map_err(|E| E.to_string())
}
