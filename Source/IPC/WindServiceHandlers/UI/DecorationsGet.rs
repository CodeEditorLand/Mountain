
//! Wire method: `decorations:get`.
//! Reads a URI decoration from cache, then falls back to a registered
//! FileDecoration provider via Cocoon gRPC.

use std::sync::Arc;

use serde_json::Value;
use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:get requires uri".to_string())?;

	if let Some(Cached) = RunTime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
		return Ok(Cached);
	}

	if let Ok(ParsedUri) = url::Url::parse(Uri) {
		match RunTime.Environment.ProvideFileDecoration(ParsedUri).await {
			Ok(Some(Result)) => {
				RunTime
					.Environment
					.ApplicationState
					.Feature
					.Decorations
					.SetDecoration(Uri, Result.clone());

				return Ok(Result);
			},

			Ok(None) => {},

			Err(E) => {
				crate::dev_log!("decorations", "warn: [DecorationsGet] provider error for {}: {}", Uri, E);
			},
		}
	}

	Ok(Value::Null)
}
