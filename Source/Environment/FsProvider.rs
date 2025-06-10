//! Implements the `FsReader` and `FsWriter` traits for `MountainEnvironment` by
//! delegating to the logic handlers in `handlers::fs`.

use std::{path::PathBuf, sync::Arc};

use Common::{
	environment::Requires,
	error::CommonError,
	fs::{
		FsReader,
		FsWriter,
		dto::{FileSystemStatDto, FileTypeDto},
	},
};
use async_trait::async_trait;

use crate::{environment::MountainEnvironment, handlers::fs as FsHandler};

#[async_trait]
impl FsReader for MountainEnvironment {
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		FsHandler::ReadFileLogic(&self.AppHandle, Path).await
	}

	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStatDto, CommonError> {
		FsHandler::StatFileLogic(&self.AppHandle, Path).await
	}

	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, FileTypeDto)>, CommonError> {
		FsHandler::ReadDirectoryLogic(&self.AppHandle, Path).await
	}
}

#[async_trait]
impl FsWriter for MountainEnvironment {
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::WriteFileLogic(&self.AppHandle, Path, Content, Create, Overwrite).await
	}

	async fn CreateDirectory(&self, Path:&PathBuf, Recursive:bool) -> Result<(), CommonError> {
		FsHandler::CreateDirectoryLogic(&self.AppHandle, Path, Recursive).await
	}

	async fn Delete(&self, Path:&PathBuf, Recursive:bool, UseTrash:bool) -> Result<(), CommonError> {
		FsHandler::DeleteLogic(&self.AppHandle, Path, Recursive, UseTrash).await
	}

	async fn Rename(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::RenameLogic(&self.AppHandle, Source, Target, Overwrite).await
	}

	async fn Copy(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::CopyLogic(&self.AppHandle, Source, Target, Overwrite).await
	}

	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError> {
		FsHandler::CreateFileLogic(&self.AppHandle, Path).await
	}
}

impl Requires<Arc<dyn FsReader + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FsReader + Send + Sync> { Arc::new(self.clone()) }
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}
