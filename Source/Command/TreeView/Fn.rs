use serde_json::Value;
use tauri::{AppHandle, Wry};

#[tauri::command]
pub async fn TreeViewCommand(_ApplicationHandle:AppHandle<Wry>, _Method:String, _Params:Option<Value>) -> Result<Value, String> {
	Ok(Value::Null)
}
