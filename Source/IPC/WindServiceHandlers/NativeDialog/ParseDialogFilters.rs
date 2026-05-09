#![allow(non_snake_case)]
//! Parse `options.filters` from a VS Code `showOpenDialog` call into the
//! `DialogFilter` shape the Tauri dialog plugin accepts. Silently skips
//! malformed or empty entries - the user still gets the picker, just
//! without the filter hint.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::NativeDialog::DialogFilter::DialogFilter;

pub fn ParseDialogFilters(Options:&Value) -> Vec<DialogFilter> {
	Options
		.get("filters")
		.and_then(Value::as_array)
		.map(|Array| {
			Array
				.iter()
				.filter_map(|Entry| {
					let Name = Entry.get("name").and_then(Value::as_str).unwrap_or("Files").to_string();
					let Extensions:Vec<String> = Entry
						.get("extensions")
						.and_then(Value::as_array)
						.map(|List| List.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
						.unwrap_or_default();
					if Extensions.is_empty() {
						None
					} else {
						Some(DialogFilter { Name, Extensions })
					}
				})
				.collect()
		})
		.unwrap_or_default()
}
