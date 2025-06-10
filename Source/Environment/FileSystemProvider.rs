// Implements the `FileSystemReader` and `FileSystemWriter` traits for
// `MountainEnvironment` by delegating to the logic Handler in `Handler::fs`.

use std::{path::PathBuf, sync::Arc};

use Common::{
	environment::Requires,
	error::CommonError,
	fs::{
		FileSystemReader,
		FileSystemWriter,
		dto::{FileSystemStatDto, FileTypeDto},
	},
};
use async_trait::async_trait;

use crate::{Handler::fs as FsHandler, environment::MountainEnvironment};

#[async_trait]
impl FileSystemReader for MountainEnvironment {
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		FsHandler::ReadFileLogic(&self.ApplicationHandle, Path).await
	}

	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStatDto, CommonError> {
		FsHandler::StatFileLogic(&self.ApplicationHandle, Path).await
	}

	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, FileTypeDto)>, CommonError> {
		FsHandler::ReadDirectoryLogic(&self.ApplicationHandle, Path).await
	}
}

#[async_trait]
impl FileSystemWriter for MountainEnvironment {
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::WriteFileLogic(&self.ApplicationHandle, Path, Content, Create, Overwrite).await
	}

	async fn CreateDirectory(&self, Path:&PathBuf, Recursive:bool) -> Result<(), CommonError> {
		FsHandler::CreateDirectoryLogic(&self.ApplicationHandle, Path, Recursive).await
	}

	async fn Delete(&self, Path:&PathBuf, Recursive:bool, UseTrash:bool) -> Result<(), CommonError> {
		FsHandler::DeleteLogic(&self.ApplicationHandle, Path, Recursive, UseTrash).await
	}

	async fn Rename(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::RenameLogic(&self.ApplicationHandle, Source, Target, Overwrite).await
	}

	async fn Copy(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::CopyLogic(&self.ApplicationHandle, Source, Target, Overwrite).await
	}

	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError> {
		FsHandler::CreateFileLogic(&self.ApplicationHandle, Path).await
	}
}

impl Requires<Arc<dyn FileSystemReader + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader + Send + Sync> { Arc::new(self.clone()) }
}

impl Requires<Arc<dyn FileSystemWriter + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemWriter + Send + Sync> { Arc::new(self.clone()) }
}
