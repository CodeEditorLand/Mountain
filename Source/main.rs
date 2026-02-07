//! Mountain binary entry point
//!
//! This file serves as the main entry point for the Mountain application.
//! It delegates to the library's Binary module.

#[tauri::mobile_entry_point]
fn main() {
	Mountain::Binary::Main::Fn();
}
