//! `WindStorageService::Set`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};

pub fn Fn(This:&Struct, key:String, value:serde_json::Value) -> Result<(), String> {
		This.provider
			.UpdateStorageValue(false, key.to_string(), Some(value))
			.await
			.map_err(|E:CommonError| e.to_string())
	}
