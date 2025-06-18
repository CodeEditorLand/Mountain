// @module FileSystemProvider (Environment/fs)
// @description Implements the `FileSystemReader` and `FileSystemWriter` traits
// for `FileSystemEnvironment`.
// NOTE: This file is part of a legacy structure. The main implementation
// is now in the top-level `mountain/src/Environment/FileSystemProvider.rs`.

#![allow(non_snake_case)]

use std::path::PathBuf;

use async_trait::async_trait;
use Common::{
	error::CommonError,
	fs::{
		FileSystemReader,
		FileSystemWriter,
		DTO::{FileSystemStatDto, FileTypeDto},
	},
};

use super::FileSystemEnvironment;
use crate::Handler::fs as FsHandler;

#[async_trait]
impl FileSystemReader for FileSystemEnvironment {
	async fn ReadFile(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		FsHandler::ReadFileLogic(&self.ApplicationHandle, path).await
	}

	async fn StatFile(&self, path:&PathBuf) -> Result<FileSystemStatDto, CommonError> {
		FsHandler::StatFileLogic(&self.ApplicationHandle, path).await
	}

	async fn ReadDirectory(&self, path:&PathBuf) -> Result<Vec<(String, FileTypeDto)>, CommonError> {
		FsHandler::ReadDirectoryLogic(&self.ApplicationHandle, path).await
	}
}

#[async_trait]
impl FileSystemWriter for FileSystemEnvironment {
	async fn WriteFile(&self, path:&PathBuf, content:Vec<u8>, create:bool, overwrite:bool) -> Result<(), CommonError> {
		FsHandler::WriteFileLogic(&self.ApplicationHandle, path, content, create, overwrite).await
	}

	async fn CreateDirectory(&self, path:&PathBuf, recursive:bool) -> Result<(), CommonError> {
		FsHandler::CreateDirectoryLogic(&self.ApplicationHandle, path, recursive).await
	}

	async fn Delete(&self, path:&PathBuf, recursive:bool, use_trash:bool) -> Result<(), CommonError> {
		FsHandler::DeleteLogic(&self.ApplicationHandle, path, recursive, use_trash).await
	}

	async fn Rename(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		FsHandler::RenameLogic(&self.ApplicationHandle, source, target, overwrite).await
	}

	async fn Copy(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		FsHandler::CopyLogic(&self.ApplicationHandle, source, target, overwrite).await
	}

	async fn CreateFile(&self, path:&PathBuf) -> Result<(), CommonError> {
		FsHandler::CreateFileLogic(&self.ApplicationHandle, path).await
	}
}
