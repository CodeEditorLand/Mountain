#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Navigation history and label handlers.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

// ============================================================================
// Navigation History Handlers
// ============================================================================

/// Navigate backward in the editor history stack.
pub async fn HistoryGoBack(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = runtime.Environment.ApplicationState.Feature.NavigationHistory.GoBack();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}

/// Navigate forward in the editor history stack.
pub async fn HistoryGoForward(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = runtime.Environment.ApplicationState.Feature.NavigationHistory.GoForward();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}

/// Return whether backward navigation is available.
pub async fn HistoryCanGoBack(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		runtime.Environment.ApplicationState.Feature.NavigationHistory.CanGoBack(),
	))
}

/// Return whether forward navigation is available.
pub async fn HistoryCanGoForward(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		runtime.Environment.ApplicationState.Feature.NavigationHistory.CanGoForward(),
	))
}

/// Push a URI onto the navigation history stack.
pub async fn HistoryPush(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("history:push requires uri".to_string())?
		.to_owned();

	runtime.Environment.ApplicationState.Feature.NavigationHistory.Push(Uri);
	Ok(Value::Null)
}

/// Clear the entire navigation history stack.
pub async fn HistoryClear(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	runtime.Environment.ApplicationState.Feature.NavigationHistory.Clear();
	Ok(Value::Null)
}

/// Return the full navigation history stack as an array of URI strings.
pub async fn HistoryGetStack(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Stack = runtime.Environment.ApplicationState.Feature.NavigationHistory.GetStack();
	Ok(Value::Array(Stack.into_iter().map(Value::String).collect()))
}

// ============================================================================
// Label Handlers
// ============================================================================

/// Resolve a human-readable display label for a URI.
///
/// Args: [uri: string, relative: bool]
/// Returns: string label
pub async fn LabelGetURI(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getUri requires uri".to_string())?
		.to_owned();

	let Relative = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	if !Relative {
		// Absolute: strip file:// scheme if present, return raw path
		let Label = if Uri.starts_with("file://") {
			Uri.trim_start_matches("file://").to_owned()
		} else {
			Uri.clone()
		};
		return Ok(Value::String(Label));
	}

	// Relative: make path relative to workspace root if possible
	let WorkspaceRoot = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.URI.to_string())
		.unwrap_or_default();

	let RawPath = if Uri.starts_with("file://") {
		Uri.trim_start_matches("file://").to_owned()
	} else {
		Uri.clone()
	};

	let RootPath = if WorkspaceRoot.starts_with("file://") {
		WorkspaceRoot.trim_start_matches("file://").to_owned()
	} else {
		WorkspaceRoot
	};

	let Label = if !RootPath.is_empty() && RawPath.starts_with(&RootPath) {
		RawPath[RootPath.len()..].trim_start_matches('/').to_owned()
	} else {
		RawPath
	};

	Ok(Value::String(Label))
}

/// Return the display label for the current workspace root folder.
pub async fn LabelGetWorkspace(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Label = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| {
			if !F.Name.is_empty() {
				F.Name
			} else {
				F.URI
					.path_segments()
					.and_then(|mut S| S.next_back())
					.map(|S| S.to_owned())
					.unwrap_or_else(|| F.URI.to_string())
			}
		})
		.unwrap_or_default();

	Ok(Value::String(Label))
}

/// Return only the basename (filename + extension) of a URI.
pub async fn LabelGetBase(args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getBase requires uri".to_string())?;

	let Base = Uri.split('/').next_back().unwrap_or(Uri);
	Ok(Value::String(Base.to_owned()))
}
