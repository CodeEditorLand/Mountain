//! `nativeHost:showOpenDialog` handler. Wires VS Code's
//! `nativeHostService.showOpenDialog(options)` contract to Tauri's
//! dialog plugin.
//!
//! Contract:
//!   - `properties: ["openDirectory" | "openFile" | "multiSelections" |
//!     "createDirectory" | "showHiddenFiles"]`
//!   - `filters: [{ name, extensions: ["vsix", …] }, …]`
//!   - `title`, `buttonLabel`, `defaultPath`
//!   - returns `{ canceled: bool, filePaths: string[] }`.
//!
//! The "Install from VSIX…" flow relies on `filters` to narrow the picker
//! to `.vsix` and on `openFile + multiSelections` so the user can pick
//! multiple archives at once.

use serde_json::{Value, json};

use tauri::AppHandle;

use crate::{IPC::WindServiceHandlers::NativeDialog::ParseDialogFilters::Fn as ParseDialogFilters, dev_log};

pub async fn Fn(ApplicationHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {

	use tauri_plugin_dialog::DialogExt;

	dev_log!("folder", "showOpenDialog: {:?}", Args);

	// Electron passes `(windowId, options)`; `options` is always the last
	// element regardless of how the renderer was invoked. Searching by
	// shape (`first object with a 'properties' or 'filters' field`) keeps
	// us robust against VS Code versions that pass an extra prefix arg.
	let Options = Args.iter().rev().find(|V| V.is_object()).cloned().unwrap_or(Value::Null);

	let Properties:Vec<String> = Options
		.get("properties")
		.and_then(Value::as_array)
		.map(|Array| Array.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
		.unwrap_or_default();

	let IsFolder = Properties.iter().any(|P| P == "openDirectory");

	let IsMultiple = Properties.iter().any(|P| P == "multiSelections");

	let Title = Options
		.get("title")
		.and_then(Value::as_str)
		.unwrap_or(if IsFolder { "Open Folder" } else { "Open File" })
		.to_string();

	let DefaultPath = Options.get("defaultPath").and_then(Value::as_str).map(str::to_string);

	let Filters = ParseDialogFilters(&Options);

	let Handle = ApplicationHandle.clone();

	let FiltersForThread = Filters.clone();

	let Selected = tokio::task::spawn_blocking(move || -> Vec<String> {
		let mut Builder = Handle.dialog().file().set_title(&Title);

		if let Some(Path) = DefaultPath.as_deref() {
			Builder = Builder.set_directory(Path);
		}

		// Apply filters only for file pickers - Tauri returns an error on
		// folder pickers if filters are set on some platforms.
		if !IsFolder {
			for Filter in &FiltersForThread {
				let ExtRefs:Vec<&str> = Filter.Extensions.iter().map(String::as_str).collect();

				Builder = Builder.add_filter(&Filter.Name, &ExtRefs);
			}
		}

		if IsFolder {
			if IsMultiple {
				Builder
					.blocking_pick_folders()
					.unwrap_or_default()
					.into_iter()
					.map(|P| P.to_string())
					.collect()
			} else {
				Builder.blocking_pick_folder().map(|P| vec![P.to_string()]).unwrap_or_default()
			}
		} else if IsMultiple {
			Builder
				.blocking_pick_files()
				.unwrap_or_default()
				.into_iter()
				.map(|P| P.to_string())
				.collect()
		} else {
			Builder.blocking_pick_file().map(|P| vec![P.to_string()]).unwrap_or_default()
		}
	})
	.await
	.map_err(|Error| format!("showOpenDialog join error: {}", Error))?;

	if Selected.is_empty() {
		dev_log!("folder", "showOpenDialog cancelled by user");

		Ok(json!({ "canceled": true, "filePaths": [] }))
	} else {
		dev_log!(
			"folder",

			"showOpenDialog selected {} path(s) (folder={}, multi={}, filters={})",

			Selected.len(),

			IsFolder,

			IsMultiple,

			Filters.len()
		);

		Ok(json!({ "canceled": false, "filePaths": Selected }))
	}
}
