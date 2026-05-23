#![allow(unused_variables)]
//! File decoration handlers (URI → badge / tooltip / colour) backing
//! `vscode.window.registerFileDecorationProvider`. Mountain's
//! `ApplicationState::Feature::Decorations` owns the map keyed on URI
//! string; handlers here just read / mutate that store.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn DecorationsGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:get requires uri".to_string())?;

	// 1. Check the static in-memory store (populated by decorations:set or prior
	//    provider calls).
	if let Some(Cached) = RunTime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
		return Ok(Cached);
	}

	// 2. Ask registered FileDecoration providers via Cocoon gRPC so
	//    extension-contributed decorations (e.g. git status badges from vscode.git,
	//    error badges from eslint) populate on demand.
	if let Ok(ParsedUri) = url::Url::parse(Uri) {
		match RunTime.Environment.ProvideFileDecoration(ParsedUri).await {
			Ok(Some(Result)) => {
				// Cache for subsequent requests so the next `decorations:get`
				// for the same URI doesn't round-trip again.
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

pub async fn DecorationsGetMany(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uris:Vec<String> = Arguments
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| Arr.iter().filter_map(|U| U.as_str().map(str::to_owned)).collect())
		.unwrap_or_default();

	let mut Result = serde_json::Map::new();

	for Uri in &Uris {
		if let Some(Decoration) = RunTime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
			Result.insert(Uri.clone(), Decoration);
		}
	}

	Ok(Value::Object(Result))
}

pub async fn DecorationsSet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:set requires uri".to_string())?;

	let Decoration = Arguments.get(1).cloned().unwrap_or(Value::Null);

	RunTime
		.Environment
		.ApplicationState
		.Feature
		.Decorations
		.SetDecoration(Uri, Decoration);

	Ok(Value::Null)
}

pub async fn DecorationsClear(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:clear requires uri".to_string())?;

	RunTime.Environment.ApplicationState.Feature.Decorations.ClearDecoration(Uri);

	Ok(Value::Null)
}
