use std::path::PathBuf;

use Common::{
	error::CommonError,
	fs::{
		FsReader,
		FsWriter,
		dto::{FileSystemStatDto, FileTypeDto},
	},
};
use async_trait::async_trait;

/// @module FsProvider (Environment/fs)
/// @description Implements the `FsReader` and `FsWriter` traits for
/// `FsEnvironment`.
use super::FsEnvironment;
use crate::handlers::fs as FsHandler;

#[async_trait]
impl FsReader for FsEnvironment {
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		FsHandler::ReadFileLogic(&self.AppHandle, Path).await
	}
	// ... other FsReader delegations
}

#[async_trait]
impl FsWriter for FsEnvironment {
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError> {
		FsHandler::WriteFileLogic(&self.AppHandle, Path, Content, Create, Overwrite).await
	}
	// ... other FsWriter delegations
}
