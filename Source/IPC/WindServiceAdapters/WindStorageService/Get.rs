//! `WindStorageService::Get`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};

pub fn Fn(This:&Struct, key:String) -> Result<serde_json::Value, String> {
		let value = self
			.provider
			.GetStorageValue(false, &key)
			.await
			.map_err(|E:CommonError| e.to_string())?
			.ok_or_else(|| "Storage key not found".to_string())?;

		Ok(value)
	}
