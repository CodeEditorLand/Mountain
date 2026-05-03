#![allow(non_snake_case)]

//! Mountain's own sandbox-config payload (input to
//! `WindServiceAdapter::convert_to_wind_configuration`).
//! Private to this module; the trio of nested DTOs
//! (`Versions`, `NLSConfiguration`, `ProductConfiguration`)
//! lives inline because they're consumed only here and never
//! constructed externally.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Struct {
	pub window_id:String,
	pub machine_id:String,
	pub session_id:String,
	pub log_level:i32,
	pub user_env:HashMap<String, String>,
	pub app_root:String,
	pub app_name:String,
	pub app_uri_scheme:String,
	pub app_language:String,
	pub app_host:String,
	pub platform:String,
	pub arch:String,
	pub versions:Versions,
	pub exec_path:String,
	pub home_dir:String,
	pub tmp_dir:String,
	pub user_data_dir:String,
	pub backup_path:String,
	pub resources_path:String,
	pub vscode_cwd:String,
	pub nls:NLSConfiguration,
	pub product_configuration:ProductConfiguration,
	pub zoom_level:f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Versions {
	pub mountain:String,
	pub electron:String,
	pub chrome:String,
	pub node:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NLSConfiguration {
	pub messages:HashMap<String, String>,
	pub language:String,
	pub available_languages:HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ProductConfiguration {
	pub name_short:String,
	pub name_long:String,
	pub application_name:String,
	pub embedder_identifier:String,
	pub is_packaged:bool,
}
