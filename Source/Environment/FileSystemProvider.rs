// @module FileSystemProvider (Environment)
// @description Implements the `FileSystemReader` and `FileSystemWriter` traits
// for `MountainEnvironment` by delegating to the logic Handler in
// `Handler::fs`.

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use Common::{
	Environment::Requires,
	error::CommonError,
	fs::{
		FileSystemReader,
		FileSystemWriter,
		DTO::{FileSystemStatDTO, FileTypeDTO},
	},
};

use super::MountainEnvironment;
use crate::Handler::fs as FileSystemHandler;

#[async_trait]
impl FileSystemReader for MountainEnvironment {
	async fn ReadFile(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		FileSystemHandler::ReadFileLogic(&self.ApplicationHandle, path).await
	}

	async fn StatFile(&self, path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
		FileSystemHandler::StatFileLogic(&self.ApplicationHandle, path).await
	}

	async fn ReadDirectory(&self, path:&PathBuf) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
		FileSystemHandler::ReadDirectoryLogic(&self.ApplicationHandle, path).await
	}
}

#[async_trait]
impl FileSystemWriter for MountainEnvironment {
	async fn WriteFile(&self, path:&PathBuf, content:Vec<u8>, create:bool, overwrite:bool) -> Result<(), CommonError> {
		FileSystemHandler::WriteFileLogic(&self.ApplicationHandle, path, content, create, overwrite).await
	}

	async fn CreateDirectory(&self, path:&PathBuf, recursive:bool) -> Result<(), CommonError> {
		FileSystemHandler::CreateDirectoryLogic(&self.ApplicationHandle, path, recursive).await
	}

	async fn Delete(&self, path:&PathBuf, recursive:bool, use_trash:bool) -> Result<(), CommonError> {
		FileSystemHandler::DeleteLogic(&self.ApplicationHandle, path, recursive, use_trash).await
	}

	async fn Rename(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		FileSystemHandler::RenameLogic(&self.ApplicationHandle, source, target, overwrite).await
	}

	async fn Copy(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		FileSystemHandler::CopyLogic(&self.ApplicationHandle, source, target, overwrite).await
	}

	async fn CreateFile(&self, path:&PathBuf) -> Result<(), CommonError> {
		FileSystemHandler::CreateFileLogic(&self.ApplicationHandle, path).await
	}
}

impl Requires<Arc<dyn FileSystemReader + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader + Send + Sync> { Arc::new(self.clone()) }
}

impl Requires<Arc<dyn FileSystemWriter + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemWriter + Send + Sync> { Arc::new(self.clone()) }
}
