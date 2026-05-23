//! Wind-shaped storage service: get / set against the
//! injected `StorageProvider` trait.

use std::sync::Arc;

use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};

pub struct Struct {
	pub(super) provider:Arc<dyn StorageProvider>,
}

impl Struct {
	pub fn new(provider:Arc<dyn StorageProvider>) -> Self { Self { provider } }

	pub async fn get(&self, key:String) -> Result<serde_json::Value, String> {
		let value = self
			.provider
			.GetStorageValue(false, &key)
			.await
			.map_err(|e:CommonError| e.to_string())?
			.ok_or_else(|| "Storage key not found".to_string())?;

		Ok(value)
	}

	pub async fn set(&self, key:String, value:serde_json::Value) -> Result<(), String> {
		self.provider
			.UpdateStorageValue(false, key.to_string(), Some(value))
			.await
			.map_err(|e:CommonError| e.to_string())
	}
}
