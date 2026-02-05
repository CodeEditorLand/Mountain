//! # CustomEditorProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements [`CustomEditorProvider`](CommonLibrary::CustomEditor::CustomEditorProvider) for [`MountainEnvironment`]
//! - Manages registration and lifecycle of custom non-text editors
//! - Coordinates Webview-based editing experiences (SVG editors, diff viewers, etc.)
//! - Handles editor resolution, save operations, and provider unregistration
//!
//! ARCHITECTURAL ROLE:
//! - Environment provider that enables extension-contributed custom editors
//! - Uses [`IPCProvider`](CommonLibrary::IPC::IPCProvider) for RPC communication with Cocoon
//! - Integrates with [`ApplicationState`](crate::ApplicationState::ApplicationState)
//!   for provider registration persistence
//!
//! ERROR HANDLING:
//! - Uses [`CommonError`](CommonLibrary::Error::CommonError) for all operations
//! - ViewType validation: rejects empty view types with InvalidArgument error
//! - Some operations are stubbed with logging/warning (OnSaveCustomDocument)
//!
//! PERFORMANCE:
//! - Provider registration lookup should be O(1) via hash map in ApplicationState (TODO)
//! - ResolveCustomEditor uses fire-and-forget RPC pattern to avoid waiting
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/contrib/customEditor/browser/customEditorService.ts` - custom editor service
//! - `vs/workbench/contrib/customEditor/common/customEditor.ts` - custom editor interfaces
//! - `vs/platform/workspace/common/workspace.ts` - resource URI handling
//!
//! TODO:
//! - Store provider registrations in ApplicationState with capability metadata
//! - Implement custom editor backup/restore mechanism
//! - Add support for multiple active instances of the same viewType
//! - Implement custom editor move and rename handling
//! - Add proper validation of viewType and resource URI
//! - Implement editor-specific command registration
//! - Add support for custom editor dispose/cleanup
//! - Consider adding editor state persistence across reloads
//! - Implement proper error recovery for Webview crashes
//! - Add telemetry for custom editor usage metrics
//!
//! MODULE CONTENTS:
//! - [`CustomEditorProvider`](CommonLibrary::CustomEditor::CustomEditorProvider) implementation:
//!   - [`RegisterCustomEditorProvider`](Self::RegisterCustomEditorProvider) - register extension provider
//!   - [`UnregisterCustomEditorProvider`](Self::UnregisterCustomEditorProvider) - unregister provider
//!   - [`OnSaveCustomDocument`](Self::OnSaveCustomDocument) - save handler (stub)
//!   - [`ResolveCustomEditor`](Self::ResolveCustomEditor) - resolve editor content via RPC

use std::sync::Arc;

use CommonLibrary::{
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
	async fn RegisterCustomEditorProvider(&self, ViewType:String, Options:Value) -> Result<(), CommonError> {
		info!("[CustomEditorProvider] Registering provider for view type: {}", ViewType);

		// Validate ViewType is non-empty
		if ViewType.is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"ViewType".to_string(),
				Reason:"ViewType cannot be empty".to_string(),
			});
		}

		// TODO: Store provider registration in ApplicationState
		// - Associate ViewType with sidecar identifier
		// - Store provider capabilities (supportsMultipleEditors, etc.)
		// - Store custom options for the provider
		// - Validate that viewType is not already registered

		Ok(())
	}

	async fn UnregisterCustomEditorProvider(&self, ViewType:String) -> Result<(), CommonError> {
		info!("[CustomEditorProvider] Unregistering provider for view type: {}", ViewType);

		// TODO: Remove provider registration from ApplicationState
		// - Check if any active editors are using this viewType
		// - Optionally close active editors or show warning

		Ok(())
	}

	async fn OnSaveCustomDocument(&self, ViewType:String, ResourceURI:Url) -> Result<(), CommonError> {
		info!(
			"[CustomEditorProvider] OnSaveCustomDocument called for '{}' at '{}'",
			ViewType, ResourceURI
		);

		// TODO: Implement full save flow:
		// 1. Send RPC request to extension sidecar requesting content from Webview
		// 2. Extension retrieves content from Webview via webview.postMessage
		// 3. Extension writes content back to Mountain
		// 4. Mountain persists content to file system via FileSystemWriter
		// 5. Emit save notification to UI

		warn!("[CustomEditorProvider] OnSaveCustomDocument is not fully implemented.");
		Ok(())
	}

	async fn ResolveCustomEditor(
		&self,
		ViewType:String,
		ResourceURI:Url,
		WebviewPanelHandle:String,
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
		//    etc. to populate the webview associated with the `WebviewPanelHandle`.

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();
		let ResourceURIComponents = json!({ "external": ResourceURI.to_string() });
		let RPCMethod = format!("{}$resolveCustomEditor", ProxyTarget::ExtHostCustomEditors.GetTargetPrefix());
		let RPCParameters = json!([ResourceURIComponents, ViewType, WebviewPanelHandle]);

		// This is a fire-and-forget notification. The sidecar is expected to
		// call back to the host to populate the webview.
		IPCProvider
			.SendNotificationToSideCar("cocoon-main".to_string(), RPCMethod, RPCParameters)
			.await
	}
}
