use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
// Assume Mountain has access to the real IWorkbenchExtensionEnablementService state

pub async fn handle_get_enablement_state<R: Runtime>(_app: AppHandle<R>, params: Value) -> Result<Value, String> {
    let extension_id = params.get("extensionId").and_then(|v| v.as_str()); // Extract ID
    println!("[Enablement Handler] GetState request for {:?}", extension_id);
    // TODO: Query the actual enablement service state in Mountain
    // For MVP, assume enabled if it was sent to Cocoon in the first place
    let enabled_state = 1; // Example: EnabledGlobally
    Ok(json!(enabled_state))
}
// TODO: Add handlers for setEnablement etc. if needed by shim
