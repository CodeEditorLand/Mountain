#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
	let Builder = tauri::Builder::default();

	// @TODO: FIX THIS
	// #[cfg(debug_assertions)]
	// Builder.plugin(tauri_plugin_devtools::init());

	Builder
		.plugin(tauri_plugin_shell::init())
		.run(tauri::generate_context!())
		.expect("Cannot Library.");
}
