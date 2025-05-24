#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() { Binary::Fn::Fn(); }

pub mod Binary;

// NEW:
pub mod app_state;

pub mod environment;

pub mod handlers {
	pub mod commands;

	pub mod config;

	pub mod diagnostics;

	pub mod documents;

	pub mod enablement;

	pub mod native_fs;

	pub mod output;

	pub mod process_mgmt;

	pub mod protocol;

	pub mod proxy;

	pub mod registry;

	pub mod secrets;

	pub mod storage;

	pub mod terminal;

	pub mod ui;

	pub mod workspace;

	pub mod workspace_fs_api;
}

pub mod Entry;

pub mod mist;

pub mod track;

pub mod rpc;

pub mod runtime;

pub mod vine;
