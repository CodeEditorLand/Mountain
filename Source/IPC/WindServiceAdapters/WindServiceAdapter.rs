#![allow(non_snake_case)]

//! Bridge between Mountain's runtime and Wind's expected
//! service interfaces. `convert_to_wind_configuration` turns
//! Mountain's sandbox config into the
//! `WindDesktopConfiguration::Struct` Sky deserialises;
//! `get_*_service` factories produce the per-domain wrappers
//! (env, file, storage, configuration) Wind needs.

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

pub struct Struct {
	pub(super) runtime:Arc<ApplicationRunTime>,
}

impl Struct {
	pub fn new(runtime:Arc<ApplicationRunTime>) -> Self {
		dev_log!("ipc", "[WindServiceAdapters] Creating Wind service adapter");
		Self { runtime }
	}

	pub async fn convert_to_wind_configuration(
		&self,
		mountain_config:serde_json::Value,
	) -> Result<WindDesktopConfiguration, String> {
		dev_log!("ipc", "[WindServiceAdapters] Converting Mountain config to Wind config");

		let config:MountainSandboxConfiguration = serde_json::from_value(mountain_config)
			.map_err(|e| format!("Failed to parse Mountain configuration: {}", e))?;

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

	pub async fn get_environment_service(&self) -> Result<WindEnvironmentService, String> {
		dev_log!("ipc", "[WindServiceAdapters] Getting Wind environment service");
		Ok(WindEnvironmentService::new())
	}

	pub async fn get_file_service(&self) -> Result<WindFileService, String> {
		dev_log!("ipc", "[WindServiceAdapters] Getting Wind file service");
		let file_system_reader:Arc<dyn FileSystemReader> = self.runtime.Environment.Require();
		let file_system_writer:Arc<dyn FileSystemWriter> = self.runtime.Environment.Require();
		Ok(WindFileService::new(file_system_reader, file_system_writer))
	}

	pub async fn get_storage_service(&self) -> Result<WindStorageService, String> {
		dev_log!("ipc", "[WindServiceAdapters] Getting Wind storage service");
		let storage:Arc<dyn StorageProvider> = self.runtime.Environment.Require();
		Ok(WindStorageService::new(storage))
	}

	pub async fn get_configuration_service(&self) -> Result<WindConfigurationService, String> {
		dev_log!("ipc", "[WindServiceAdapters] Getting Wind configuration service");
		let config:Arc<dyn ConfigurationProvider> = self.runtime.Environment.Require();
		Ok(WindConfigurationService::new(config))
	}
}
