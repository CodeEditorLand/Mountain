pub mod New;
pub mod ReadFile;
pub mod WriteFile;
pub mod StatFile;

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
