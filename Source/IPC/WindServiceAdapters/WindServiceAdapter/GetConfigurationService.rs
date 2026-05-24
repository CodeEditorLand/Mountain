//! `WindServiceAdapter::GetConfigurationService`

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

pub fn Fn(This:&Struct) -> Result<WindConfigurationService, String> {
		dev_log!("ipc", "[WindServiceAdapters] Getting Wind configuration service");

		let config:Arc<dyn ConfigurationProvider> = This.runtime.Environment.Require();

		Ok(WindConfigurationService::new(config))
	}
