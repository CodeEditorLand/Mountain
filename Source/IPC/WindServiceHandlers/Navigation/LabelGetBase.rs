
//! Last path segment (filename + extension) of a URI. Used by
//! the editor tabs and breadcrumbs where only the file's name
//! is wanted, not its full path.

use serde_json::Value;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getBase requires uri".to_string())?;

	let Base = Uri.split('/').next_back().unwrap_or(Uri);

	Ok(Value::String(Base.to_owned()))
}
