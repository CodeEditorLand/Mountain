//! `WindFileService::WriteFile`

use super::Struct;
use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::json;

pub fn Fn(This:&Struct, path:String, content:Vec<u8>) -> Result<(), String> {
		This.writer
			.WriteFile(&PathBuf::from(path), content, true, true)
			.await
			.map_err(|E:CommonError| e.to_string())
	}
