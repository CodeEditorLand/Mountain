// File: Mountain/Source/Environment/CustomEditorProvider.rs
//
// # Architectural Role: Custom Editor Lifecycle Management
//
// CustomEditorProvider implements the CustomEditorProvider trait, managing the
// lifecycle of custom non-text editors contributed by extensions. These editors
// use Webview panels to provide specialized editing experiences (e.g., SVG
// editors, diff viewers, image editors).
//
// # Responsibilities
//
// 1. **Provider Registration**: Manages registration of custom editor
//    providers, each identified by a unique viewType.
//
// 2. **Editor Orchestration**: Coordinates between the UI (Webview), the
//    extension host (Cocoon), and the filesystem to provide a seamless editing
//    experience.
//
// 3. **Content Resolution**: Mediates the "resolve" process where the extension
//    provides initial content and HTML for the custom editor.
//
// 4. **Lifecycle Events**: Handles registration, unregistration, save
//    operations, and editor lifecycle events.
//
// # Custom Editor Flow
//
// 1. Extension registers a custom editor provider via
//    RegisterCustomEditorProvider
// 2. UI requests to open a resource with a custom viewType
// 3. Mountain calls ResolveCustomEditor with viewType, resource URI, and
//    Webview handle
// 4. Extension receives RPC call and provides HTML/content for the Webview
// 5. Extension can send messages back and forth via Webview communication
// 6. On save, Mountain calls OnSaveCustomDocument to persist changes
//
// # Patterns Borrowed from VSCode
//
// - **Webview API**: Inspired by VSCode's WebviewPanel API for custom editors.
//
// - **Content Providers**: Similar to VSCode's TextDocumentContentProvider
//   pattern for providing content with custom URI schemes.
//
// - **Extension Contribution**: Follows VSCode's contribution pattern where
//   extensions declare custom editors in package.json.
//
// # TODOs
//
// - [ ] Store provider registrations in ApplicationState with capability
//   metadata
// - [ ] Implement custom editor backup/restore mechanism
// - [ ] Add support for multiple active instances of the same viewType
// - [ ] Implement custom editor move and rename handling
// - [ ] Add proper validation of viewType and resource URI
// - [ ] Implement editor-specific command registration
// - [ ] Add support for custom editor dispose/cleanup
// - [ ] Consider adding editor state persistence across reloads
// - [ ] Implement proper error recovery for Webview crashes
// - [ ] Add telemetry for custom editor usage metrics

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
