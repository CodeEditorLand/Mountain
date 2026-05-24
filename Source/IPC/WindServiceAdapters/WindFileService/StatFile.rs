//! `WindFileService::StatFile`

use super::Struct;
use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::json;

pub fn Fn(This:&Struct, path:String) -> Result<serde_json::Value, String> {
		let stat_dto = self
			.reader
			.StatFile(&PathBuf::from(path))
			.await
			.map_err(|E:CommonError| e.to_string())?;

		Ok(json!(stat_dto))
	}
