#![allow(non_snake_case)]

// mod Fn;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(dead_code)]
fn main() {
	// Fn::Tauri::Fn().await;

	let Builder = tauri::Builder::default();

	Builder
		.plugin(tauri_plugin_shell::init())
		.run(tauri::generate_context!())
		.expect("Cannot Library.");
}
