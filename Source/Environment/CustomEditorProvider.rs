// File: Mountain/Source/Environment/CustomEditorProvider.rs
// Role: Implements the `CustomEditorProvider` trait for the
// `MountainEnvironment`. Responsibilities:
//   - Manage the registration and state of custom editor providers.
//   - Mediate the "resolve" process, calling back to the extension host to get
//     the content for a custom editor instance.
//
// NOTE: This is a stub implementation and needs to be fully built out.

//! # CustomEditorProvider Implementation
//!
//! Implements the `CustomEditorProvider` trait for the `MountainEnvironment`.
//! This is currently a stub implementation.

use Common::{CustomEditor::CustomEditorProvider::CustomEditorProvider, Error::CommonError::CommonError};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;
use url::Url;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl CustomEditorProvider for MountainEnvironment {
	async fn RegisterCustomEditorProvider(&self, _ViewType:String, _OptionsValue:Value) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] RegisterCustomEditorProvider is not implemented.");

		// TODO: Store provider info in ApplicationState.
		Ok(())
	}

	async fn UnregisterCustomEditorProvider(&self, _ViewType:String) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] UnregisterCustomEditorProvider is not implemented.");

		// TODO: Remove provider info from ApplicationState.
		Ok(())
	}

	async fn OnSaveCustomDocument(&self, _ViewType:String, _ResourceURI:Url) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] OnSaveCustomDocument is not implemented.");

		// This would typically trigger a call to the extension host to perform the
		// save.
		Ok(())
	}

	async fn ResolveCustomEditor(
		&self,

		_ViewType:String,

		_ResourceURI:Url,

		_WebViewPanelHandle:String,
	) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] ResolveCustomEditor is not implemented.");

		// This is the core logic:
		// 1. Find the sidecar that registered this ViewType.
		// 2. Make an RPC call to that sidecar's implementation of
		//    `$resolveCustomEditor`.
		// 3. The sidecar will then call back to the host with `setHtml`, `postMessage`,
		//    etc.
		Err(CommonError::NotImplemented { FeatureName:"ResolveCustomEditor".into() })
	}
}
