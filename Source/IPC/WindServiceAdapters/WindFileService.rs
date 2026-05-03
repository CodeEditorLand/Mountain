#![allow(non_snake_case)]

//! Wind-shaped file service: read / write / stat over the
//! injected `FileSystemReader` / `FileSystemWriter` traits.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::json;

pub struct Struct {
	pub(super) reader:Arc<dyn FileSystemReader>,
	pub(super) writer:Arc<dyn FileSystemWriter>,
}

impl Struct {
	pub fn new(reader:Arc<dyn FileSystemReader>, writer:Arc<dyn FileSystemWriter>) -> Self { Self { reader, writer } }

	pub async fn read_file(&self, path:String) -> Result<Vec<u8>, String> {
		self.reader.ReadFile(&PathBuf::from(path)).await.map_err(|e| e.to_string())
	}

	pub async fn write_file(&self, path:String, content:Vec<u8>) -> Result<(), String> {
		self.writer
			.WriteFile(&PathBuf::from(path), content, true, true)
			.await
			.map_err(|e:CommonError| e.to_string())
	}

	pub async fn stat_file(&self, path:String) -> Result<serde_json::Value, String> {
		let stat_dto = self
			.reader
			.StatFile(&PathBuf::from(path))
			.await
			.map_err(|e:CommonError| e.to_string())?;
		Ok(json!(stat_dto))
	}
}
