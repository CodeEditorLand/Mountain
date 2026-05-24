//! `WindServiceAdapter::ConvertToWindConfiguration`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::{
	Configuration::ConfigurationProvider::ConfigurationProvider,
	Environment::Requires::Requires,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	Storage::StorageProvider::StorageProvider,
};
use crate::{
	IPC::WindServiceAdapters::{
		MountainSandboxConfiguration::Struct as MountainSandboxConfiguration,
		OsInfo::Struct as OsInfo,
		Profiles::Struct as Profiles,
		WindConfigurationService::Struct as WindConfigurationService,
		WindDesktopConfiguration::Struct as WindDesktopConfiguration,
		WindEnvironmentService::Struct as WindEnvironmentService,
		WindFileService::Struct as WindFileService,
		WindStorageService::Struct as WindStorageService,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub fn Fn(
		&self,

		mountain_config:serde_json::Value,
	) -> Result<WindDesktopConfiguration, String> {
		dev_log!("ipc", "[WindServiceAdapters] Converting Mountain config to Wind config");

		let config:MountainSandboxConfiguration = serde_json::from_value(mountain_config)
			.map_err(|E| format!("Failed to parse Mountain configuration: {}", e))?;

		Ok(WindDesktopConfiguration {
			window_id:config.window_id.parse().unwrap_or(1),
			app_root:config.app_root,
			user_data_path:config.user_data_dir,
			temp_path:config.tmp_dir,
			log_level:config.log_level.to_string(),
			is_packaged:config.product_configuration.is_packaged,
			tauri_version:config.versions.mountain,
			platform:config.platform,
			arch:config.arch,
			workspace:None,
			files_to_open_or_create:None,
			files_to_diff:None,
			files_to_wait:None,
			fullscreen:Some(false),
			zoom_level:Some(config.zoom_level),
			is_custom_zoom_level:Some(false),
			profiles:Profiles { all:vec![], home:config.home_dir, profile:serde_json::Value::Null },
			policies_data:None,
			loggers:vec![],
			backup_path:Some(config.backup_path),
			disable_layout_restore:Some(false),
			os:OsInfo { release:std::env::consts::OS.to_string() },
		})
	}
