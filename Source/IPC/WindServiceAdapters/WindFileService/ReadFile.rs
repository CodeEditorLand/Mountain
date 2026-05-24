//! `WindFileService::ReadFile`

use super::Struct;
use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::json;

pub fn Fn(This:&Struct, path:String) -> Result<Vec<u8>, String> {
		This.reader.ReadFile(&PathBuf::from(path)).await.map_err(|E| e.to_string())
	}
