pub fn Matches(MethodName:&str) -> bool {

	MethodName.starts_with("$statusBar:")

		|| MethodName == "$setStatusBarMessage"
		|| MethodName == "$disposeStatusBarMessage"
}

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};

use serde_json::{Value, json};

use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{obj_bool, obj_f64, obj_str, string_at},
	MappedEffectType::MappedEffect,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {

	match MethodName {
		"$statusBar:set" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

				// The extension host serialises this as an object with named fields,
				// matching the shape used by the RPC layer (EntryIdentifier = id).
				let Parameters = Parameters.clone();

				let entry_id = Parameters
					.get("id")
					.and_then(Value::as_str)
					.ok_or_else(|| "$statusBar:set: missing 'id' field".to_string())?;

				let item_id = Parameters.get("itemId").and_then(Value::as_str).unwrap_or(entry_id);

				let ext_id = obj_str(&Parameters, "extensionId");

				let text = obj_str(&Parameters, "text").to_string();

				let tooltip = Parameters.get("tooltip").cloned();

				let command = Parameters.get("command").cloned();

				let color = Parameters.get("color").cloned();

				let background_color = Parameters.get("backgroundColor").cloned();

				let is_aligned_left = obj_bool(&Parameters, "alignLeft");

				let priority = obj_f64(&Parameters, "priority");

				let accessibility = Parameters.get("accessibilityInformation").cloned();

				let entry = StatusBarEntryDTO {
					EntryIdentifier:entry_id.to_string(),
					ItemIdentifier:item_id.to_string(),
					ExtensionIdentifier:ext_id.to_string(),
					Name:None,
					Text:text,
					Tooltip:tooltip,
					HasTooltipProvider:false,
					Command:command,
					Color:color,
					BackgroundColor:background_color,
					IsAlignedLeft:is_aligned_left,
					Priority:priority,
					AccessibilityInformation:accessibility,
				};

				provider
					.SetStatusBarEntry(entry)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$statusBar:dispose" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

				// Require a non-empty id - a missing id would silently target the
				// wrong entry (previously fell back to the literal string "id").
				let id = Parameters
					.get(0)
					.and_then(Value::as_str)
					.filter(|s| !s.is_empty())
					.ok_or_else(|| "$statusBar:dispose: missing or empty entry id".to_string())?;

				provider
					.DisposeStatusBarEntry(id.to_string())
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		// Cocoon's `Window/Status/Bar.ts` sends `setStatusBarText` as a
		// fire-and-forget notification when an extension's StatusBarItem
		// text mutates (`set text(...)`). It carries `{ itemId, text }`
		// only - the rest of the entry (id, alignment, priority, etc.)
		// was set when the item was first created. Forward to the
		// StatusBarProvider's text-only mutation path. Without this arm,
		// status bar text never updates after the initial create.
		"setStatusBarText" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

				let item_id = obj_str(&Parameters, "itemId");

				if item_id.is_empty() {
					return Err("setStatusBarText: missing 'itemId' field".to_string());
				}

				let text = obj_str(&Parameters, "text").to_string();

				// Re-use the text-only message path (same semantics as
				// `$setStatusBarMessage`). The item lives under itemId
				// from the prior `$statusBar:set` registration.
				provider
					.SetStatusBarMessage(item_id.to_string(), text)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$setStatusBarMessage" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

				let message_id = Parameters
					.get(0)
					.and_then(Value::as_str)
					.filter(|s| !s.is_empty())
					.ok_or_else(|| "$setStatusBarMessage: missing or empty message id".to_string())?;

				let text = string_at(&Parameters, 1);

				provider
					.SetStatusBarMessage(message_id.to_string(), text)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$disposeStatusBarMessage" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

				let message_id = Parameters
					.get(0)
					.and_then(Value::as_str)
					.filter(|s| !s.is_empty())
					.ok_or_else(|| "$disposeStatusBarMessage: missing or empty message id".to_string())?;

				provider
					.DisposeStatusBarMessage(message_id.to_string())
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
