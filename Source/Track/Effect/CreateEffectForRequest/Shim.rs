//! # Shim domain module for CreateEffectForRequest
//!
//! Gets first crack at EVERY gRPC method before any other domain module.
//! Gated behind `TierShim` (default: `None` — zero overhead).
//! When the shim is active and SwallowMap matches, the method is
//! swallowed here; otherwise falls through to the rest of the Try! chain.

use crate::Shim::{Gate, SwallowMap};
use serde_json::Value;
use tauri::Runtime;
use crate::Track::Effect::MappedEffectType::MappedEffect;

/// Always true when the shim gate is active — gives this module
/// priority over every other domain module in the Try! chain.
pub fn Matches(_method: &str) -> bool {
	Gate::is_enabled()
}

/// If SwallowMap decides this method should be swallowed,
/// return a Null-ack effect. Otherwise return `None` so the
/// rest of the Try! chain can handle it.
pub fn CreateEffect<R: Runtime>(method: &str, _params: Value) -> Option<Result<MappedEffect, String>> {
	if SwallowMap::should_swallow(method) {
		let effect = move |_run_time: std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>|
			-> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>>
		{
			Box::pin(async move { Ok(serde_json::json!(null)) })
		};
		Some(Ok(Box::new(effect)))
	} else {
		None // let other domain modules handle it
	}
}
