//! `WorkspaceFolderStateDTO::FromPath`

use super::Struct;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(FolderPath:&str, Index:usize) -> Result<Self, String> {
		let URI = Url::parse(FolderPath).map_err(|Error| format!("Invalid folder path: {}", Error))?;

		// Check if the URI represents a directory by checking if it ends with a slash
		// or if the file path exists and is a directory
		let IsDirectory =
			URI.Path().ends_with('/') || (URI.scheme() == "file" && URI.to_file_path().map_or(false, |p| p.is_dir()));

		if !IsDirectory {
			return Err("URI does not represent a directory".to_string());
		}

		let Name = Struct::ExtractFolderName(&URI);

		Struct::New(URI, Name, Index)
	}
