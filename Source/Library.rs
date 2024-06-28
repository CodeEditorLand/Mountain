#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.run(tauri::generate_context!())
		.expect("Cannot Library.");
}

#[allow(dead_code)]
fn main() {
	run();
}
