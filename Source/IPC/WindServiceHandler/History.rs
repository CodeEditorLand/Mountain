#![allow(non_snake_case)]

//! Navigation History domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Navigate backward in the editor history stack.
pub async fn handle_history_go_back(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = Runtime.Environment.ApplicationState.Feature.NavigationHistory.GoBack();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}

/// Navigate forward in the editor history stack.
pub async fn handle_history_go_forward(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = Runtime.Environment.ApplicationState.Feature.NavigationHistory.GoForward();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}

/// Return whether backward navigation is available.
pub async fn handle_history_can_go_back(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		Runtime.Environment.ApplicationState.Feature.NavigationHistory.CanGoBack(),
	))
}

/// Return whether forward navigation is available.
pub async fn handle_history_can_go_forward(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		Runtime.Environment.ApplicationState.Feature.NavigationHistory.CanGoForward(),
	))
}

/// Push a URI onto the navigation history stack.
pub async fn handle_history_push(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("history:push requires uri".to_string())?
		.to_owned();

	Runtime.Environment.ApplicationState.Feature.NavigationHistory.Push(Uri);
	Ok(Value::Null)
}

/// Clear the entire navigation history stack.
pub async fn handle_history_clear(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Runtime.Environment.ApplicationState.Feature.NavigationHistory.Clear();
	Ok(Value::Null)
}

/// Return the full navigation history stack as an array of URI strings.
pub async fn handle_history_get_stack(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Stack = Runtime.Environment.ApplicationState.Feature.NavigationHistory.GetStack();
	Ok(Value::Array(Stack.into_iter().map(Value::String).collect()))
}
