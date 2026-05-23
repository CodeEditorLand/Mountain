
//! Mirror of Wind's `IDesktopConfiguration` interface - the
//! shape Sky deserialises on boot. Built by
//! `WindServiceAdapter::convert_to_wind_configuration` from
//! Mountain's sandbox config.

use serde::{Deserialize, Serialize};

use crate::IPC::WindServiceAdapters::{FileToDiff, FileToOpenOrCreate, FilesToWait, Logger, OsInfo, Profiles};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub window_id:u32,

	pub app_root:String,

	pub user_data_path:String,

	pub temp_path:String,

	pub log_level:String,

	pub is_packaged:bool,

	pub tauri_version:String,

	pub platform:String,

	pub arch:String,

	pub workspace:Option<serde_json::Value>,

	pub files_to_open_or_create:Option<Vec<FileToOpenOrCreate::Struct>>,

	pub files_to_diff:Option<Vec<FileToDiff::Struct>>,

	pub files_to_wait:Option<FilesToWait::Struct>,

	pub fullscreen:Option<bool>,

	pub zoom_level:Option<f64>,

	pub is_custom_zoom_level:Option<bool>,

	pub profiles:Profiles::Struct,

	pub policies_data:Option<serde_json::Value>,

	pub loggers:Vec<Logger::Struct>,

	pub backup_path:Option<String>,

	pub disable_layout_restore:Option<bool>,

	pub os:OsInfo::Struct,
}
