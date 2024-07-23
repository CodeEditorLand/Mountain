#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
	Fn::Binary::Fn();
}

pub mod Fn;
pub mod Struct;
