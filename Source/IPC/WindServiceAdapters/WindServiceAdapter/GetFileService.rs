//! `WindServiceAdapter::GetFileService`

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

pub fn Fn(This:&Struct) -> Result<WindFileService, String> {
		dev_log!("ipc", "[WindServiceAdapters] Getting Wind file service");

		let file_system_reader:Arc<dyn FileSystemReader> = This.runtime.Environment.Require();

		let file_system_writer:Arc<dyn FileSystemWriter> = This.runtime.Environment.Require();

		Ok(WindFileService::new(file_system_reader, file_system_writer))
	}
