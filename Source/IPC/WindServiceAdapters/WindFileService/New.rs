//! `WindFileService::New`

use super::Struct;
use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::json;

pub fn Fn(reader:Arc<dyn FileSystemReader>, writer:Arc<dyn FileSystemWriter>) -> Struct { Self { reader, writer } }
