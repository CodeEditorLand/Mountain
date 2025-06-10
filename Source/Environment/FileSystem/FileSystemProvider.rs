use std::path::PathBuf;

use Common::{
	error::CommonError,
	fs::{
		FileSystemReader,
		FileSystemWriter,
		dto::{FileSystemStatDto, FileTypeDto},
	},
};
use async_trait::async_trait;

// @module FileSystemProvider (Environment/fs)
// @description Implements the `FileSystemReader` and `FileSystemWriter` traits for
// `FileSystemEnvironment`.
use super::FileSystemEnvironment;
use crate::Handler::fs as FsHandler;

#[async_trait]
impl FileSystemReader for FileSystemEnvironment {
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		FsHandler::ReadFileLogic(&self.ApplicationHandle, Path).await
	}
	// ... other FileSystemReader delegations
}

#[async_trait]
impl FileSystemWriter for FileSystemEnvironment {
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::WriteFileLogic(&self.ApplicationHandle, Path, Content, Create, Overwrite).await
	}
	// ... other FileSystemWriter delegations
}
