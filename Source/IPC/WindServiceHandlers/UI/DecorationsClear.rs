//! Wire method: `decorations:clear`.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:clear requires uri".to_string())?;

	RunTime.Environment.ApplicationState.Feature.Decorations.ClearDecoration(Uri);

	Ok(Value::Null)
}
