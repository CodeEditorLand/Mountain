//! # LanguageFeature - Invoke Provider Helper
//!
//! Generic helper to reduce boilerplate in language feature command handlers.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry};

use super::Validation::validate_language_feature_request;

/// A generic helper to reduce boilerplate in language feature command handlers.
pub(super) async fn InvokeProvider<F, T>(application_handle:AppHandle<Wry>, handler:F) -> Result<Value, String>
where
	F: FnOnce(Arc<dyn LanguageFeatureProviderRegistry>) -> T,
	T: std::future::Future<Output = Result<Value, CommonError>>, {
	let RunTime = application_handle
		.state::<Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>>()
		.inner()
		.clone();

	let Provider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();

	let result = handler(provider).await.map_err(|Error| error.to_string())?;

	serde_json::to_value(result).map_err(|Error| error.to_string())
}
