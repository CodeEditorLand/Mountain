// File: Mountain/Source/Environment/CustomEditorProvider.rs
// Role: Implements the `CustomEditorProvider` trait for the
// `MountainEnvironment`. Responsibilities:
//   - Manage the registration and state of custom editor providers.
//   - Mediate the "resolve" process, calling back to the extension host to get
//     the content for a custom editor instance.

//! # CustomEditorProvider Implementation
//!
//! Implements the `CustomEditorProvider` trait for the `MountainEnvironment`.
//! This provider orchestrates the lifecycle of custom, non-text editors that
//! are contributed by extensions.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{Value, json};
use url::Url;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl CustomEditorProvider for MountainEnvironment {
	async fn RegisterCustomEditorProvider(&self, _ViewType:String, _OptionsValue:Value) -> Result<(), CommonError> {
		info!("[CustomEditorProvider] Registering provider for view type: {}", _ViewType);
		// In a full implementation, this would store provider details, such as
		// its capabilities and the sidecar it belongs to, in ApplicationState.
		// For now, we assume all providers are in the main sidecar.
		Ok(())
	}

	async fn UnregisterCustomEditorProvider(&self, _ViewType:String) -> Result<(), CommonError> {
		info!("[CustomEditorProvider] Unregistering provider for view type: {}", _ViewType);
		// This would remove the provider's registration from ApplicationState.
		Ok(())
	}

	async fn OnSaveCustomDocument(&self, _ViewType:String, _ResourceURI:Url) -> Result<(), CommonError> {
		warn!("[CustomEditorProvider] OnSaveCustomDocument is not fully implemented.");
		// This would typically trigger a call to the extension host to perform the
		// save, which would then read data from the webview and write it to the file.
		Ok(())
	}

	async fn ResolveCustomEditor(
		&self,
		ViewType:String,
		ResourceURI:Url,
		WebViewPanelHandle:String,
	) -> Result<(), CommonError> {
		info!(
			"[CustomEditorProvider] Resolving custom editor for '{}' on resource '{}'",
			ViewType, ResourceURI
		);

		// This is the core logic:
		// 1. Find the sidecar that registered this ViewType. For now, assume
		//    "cocoon-main".
		// 2. Make an RPC call to that sidecar's implementation of
		//    `$resolveCustomEditor`.
		// 3. The sidecar will then call back to the host with `setHtml`, `postMessage`,
		//    etc. to populate the webview associated with the `WebViewPanelHandle`.

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();
		let ResourceURIComponents = json!({ "external": ResourceURI.to_string() });
		let RPCMethod = format!("{}$resolveCustomEditor", ProxyTarget::ExtHostCustomEditors.GetTargetPrefix());
		let RPCParameters = json!([ResourceURIComponents, ViewType, WebViewPanelHandle]);

		// This is a fire-and-forget notification. The sidecar is expected to
		// call back to the host to populate the webview.
		IPCProvider
			.SendNotificationToSideCar("cocoon-main".to_string(), RPCMethod, RPCParameters)
			.await
	}
}
