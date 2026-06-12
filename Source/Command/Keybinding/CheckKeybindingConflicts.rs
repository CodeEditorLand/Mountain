//! Tauri command - report every resolved keybinding that collides with a
//! candidate key expression. Key expressions are compared after
//! normalisation (`Environment::Utility::WhenClause::NormalizeKeyExpression`):
//! modifier aliases collapse (`cmd`/`meta`/`super` → `cmd`), modifier
//! order is canonicalised, and chords compare stroke-for-stroke, so
//! `"Shift+CMD+p"` conflicts with `"meta+shift+P"`.
//!
//! Mountain has no live context-key store, so two bindings with different
//! `when` clauses are still reported - the `guarded` flag tells the UI
//! whether a clause exists, and `keybinding:resolve` settles which one
//! actually fires in a given context.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::{Environment::Utility::WhenClause, RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn CheckKeybindingConflicts(ApplicationHandle:AppHandle<Wry>, Keybinding:String) -> Result<Value, String> {
	dev_log!("keybinding", "checking conflicts for keybinding: {}", Keybinding);

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	let Rules = Provider.GetResolvedKeybinding().await.map_err(|Error| Error.to_string())?;

	let Normalized = WhenClause::NormalizeKeyExpression(&Keybinding);

	let Conflicts:Vec<Value> = Rules
		.as_array()
		.map(|All| {
			All.iter()
				.filter(|Rule| {
					Rule.get("key")
						.and_then(Value::as_str)
						.is_some_and(|Key| WhenClause::NormalizeKeyExpression(Key) == Normalized)
				})
				.map(|Rule| {
					json!({
						"key": Rule.get("key").cloned().unwrap_or(Value::Null),
						"command": Rule.get("command").cloned().unwrap_or(Value::Null),
						"when": Rule.get("when").cloned().unwrap_or(Value::Null),
						"source": Rule.get("source").cloned().unwrap_or(Value::Null),
						"guarded": Rule.get("when").and_then(Value::as_str).is_some(),
					})
				})
				.collect()
		})
		.unwrap_or_default();

	Ok(json!({ "conflicts": Conflicts }))
}
