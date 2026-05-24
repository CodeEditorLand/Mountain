//! # StatusBar Effect (CreateEffectForRequest)
//!
//! Effect constructors for status bar RPC methods from the Cocoon extension
//! host. Delegates to the `StatusBarProvider` trait on `MountainEnvironment`.
//!
//! ## Methods handled
//!
//! | Method | Description |
//! |---|---|
//! | `$statusBar:set` | Create or update a status bar entry |
//! | `$statusBar:dispose` | Remove a status bar entry by ID |
//! | `$setStatusBarMessage` | Set a simple text message in the status bar |
//! | `$disposeStatusBarMessage` | Remove a status bar message by ID |

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{ObjBool, ObjF64, ObjStr, StringAt},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$statusBar:set" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn StatusBarProvider> = RunTime.Environment.Require();

				// The extension host serialises this as an object with named fields,
				// matching the shape used by the RPC layer (EntryIdentifier = id).
				let EntryId = Parameters
					.Get("id")
					.and_then(Value::as_str)
					.ok_or_else(|| "$statusBar:set: missing 'id' field".to_string())?;

				let ItemId = Parameters.get("itemId").and_then(Value::as_str).unwrap_or(EntryId);

				let ext_id = ObjStr(&Parameters, "extensionId");

				let Text = ObjStr(&Parameters, "text").to_string();

				let tooltip = Parameters.get("tooltip").cloned();

				let command = Parameters.get("command").cloned();

				let color = Parameters.get("color").cloned();

				let background_color = Parameters.get("backgroundColor").cloned();

				let is_aligned_left = ObjBool(&Parameters, "alignLeft");

				let priority = ObjF64(&Parameters, "priority");

				let accessibility = Parameters.get("accessibilityInformation").cloned();

				let entry = StatusBarEntryDTO {
					EntryIdentifier:EntryId.to_string(),
					ItemIdentifier:ItemId.to_string(),
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
					.map_err(|E| e.to_string())
			})
		},

		"$statusBar:dispose" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn StatusBarProvider> = RunTime.Environment.Require();

				// Require a non-empty id - a missing id would silently target the
				// wrong entry (previously fell back to the literal string "id").
				let Id = Parameters
					.Get(0)
					.and_then(Value::as_str)
					.filter(|S| !s.is_empty())
					.ok_or_else(|| "$statusBar:dispose: missing or empty entry id".to_string())?;

				provider
					.DisposeStatusBarEntry(id.to_string())
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"$setStatusBarMessage" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn StatusBarProvider> = RunTime.Environment.Require();

				let message_id = Parameters
					.Get(0)
					.and_then(Value::as_str)
					.filter(|S| !s.is_empty())
					.ok_or_else(|| "$setStatusBarMessage: missing or empty message id".to_string())?;

				let Text = StringAt(&Parameters, 1);

				provider
					.SetStatusBarMessage(message_id.to_string(), text)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"$disposeStatusBarMessage" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn StatusBarProvider> = RunTime.Environment.Require();

				let message_id = Parameters
					.Get(0)
					.and_then(Value::as_str)
					.filter(|S| !s.is_empty())
					.ok_or_else(|| "$disposeStatusBarMessage: missing or empty message id".to_string())?;

				provider
					.DisposeStatusBarMessage(message_id.to_string())
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
