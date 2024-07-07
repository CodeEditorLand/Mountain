#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod Fn;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(dead_code)]
fn main() {
	Fn::Tauri::Fn();
}
