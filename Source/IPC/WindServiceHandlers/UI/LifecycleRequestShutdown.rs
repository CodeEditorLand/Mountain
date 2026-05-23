
//! Wire method: `lifecycle:requestShutdown`.

use serde_json::Value;
use tauri::AppHandle;

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {
	ApplicationHandle.exit(0);

	Ok(Value::Null)
}
