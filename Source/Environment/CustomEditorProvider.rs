//! # CustomEditorProvider Implementation
//!
//! Implements the `CustomEditorProvider` trait for the `MountainEnvironment`.
//! This is currently a stub implementation.

use Common::{CustomEditor::CustomEditorProvider, Error::CommonError};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;
use url::Url;

use super::MountainEnvironment;

#[async_trait]
impl CustomEditorProvider for MountainEnvironment {
	async fn RegisterCustomEditorProvider(&self, _ViewType:String, _Options:Value) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] RegisterCustomEditorProvider is not implemented.");
		Ok(())
	}

	async fn UnregisterCustomEditorProvider(&self, _ViewType:String) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] UnregisterCustomEditorProvider is not implemented.");
		Ok(())
	}

	async fn OnSaveCustomDocument(&self, _ViewType:String, _ResourceURI:Url) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] OnSaveCustomDocument is not implemented.");
		Ok(())
	}

	async fn ResolveCustomEditor(
		&self,
		_ViewType:String,
		_ResourceURI:Url,
		_WebViewPanelHandle:String,
	) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] ResolveCustomEditor is not implemented.");
		Err(CommonError::NotImplemented { FeatureName:"ResolveCustomEditor".into() })
	}
}
