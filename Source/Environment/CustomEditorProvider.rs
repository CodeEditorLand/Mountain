//! # CustomEditorProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements
//!   [`CustomEditorProvider`](CommonLibrary::CustomEditor::CustomEditorProvider)
//!   for [`MountainEnvironment`]
//! - Manages registration and lifecycle of custom non-text editors
//! - Coordinates Webview-based editing experiences (SVG editors, diff viewers,
//!   etc.)
//! - Handles editor resolution, save operations, and provider unregistration
//!
//! ARCHITECTURAL ROLE:
//! - Environment provider that enables extension-contributed custom editors
//! - Uses [`IPCProvider`](CommonLibrary::IPC::IPCProvider) for RPC
//!   communication with Cocoon
//! - Integrates with `ApplicationState` for provider registration persistence
//!
//! ERROR HANDLING:
//! - Uses [`CommonError`](CommonLibrary::Error::CommonError) for all operations
//! - ViewType validation: rejects empty view types with InvalidArgument error
//! - OnSaveCustomDocument now reverse-RPCs to the owning sidecar via
//!   `$onSaveCustomDocument`; returns the sidecar's error verbatim on failure
//!   so the workbench's save promise rejects with a real reason.
//!
//! PERFORMANCE:
//! - Provider registration lookup should be O(1) via hash map in
//!   ApplicationState (TODO)
//! - ResolveCustomEditor uses fire-and-forget RPC pattern to avoid waiting
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/contrib/customEditor/browser/customEditorService.ts` -
//!   custom editor service
//! - `vs/workbench/contrib/customEditor/common/customEditor.ts` - custom editor
//!   interfaces
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
//! - `RegisterCustomEditorProvider` - register extension provider
//! - `UnregisterCustomEditorProvider` - unregister provider
//! - `OnSaveCustomDocument` - workbench → extension save reverse-RPC
//! - `ResolveCustomEditor` - resolve editor content via RPC

use std::sync::Arc;

use CommonLibrary::{
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::Emitter;
use url::Url;

use super::MountainEnvironment::MountainEnvironment;
use crate::dev_log;

#[async_trait]
impl CustomEditorProvider for MountainEnvironment {
	async fn RegisterCustomEditorProvider(&self, ViewType:String, _Options:Value) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[CustomEditorProvider] Registering provider for view type: {}",
			ViewType
		);

		// Validate ViewType is non-empty
		if ViewType.is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"ViewType".to_string(),
				Reason:"ViewType cannot be empty".to_string(),
			});
		}

		// Register custom editor provider in ApplicationState for lifecycle management
		// and resolution. Should associate ViewType with the sidecar identifier for
		// RPC routing, store provider capabilities (supportsMultipleEditors,
		// serialization support), store custom options (mime types, file extensions),
		// validate that the ViewType is not already registered to prevent conflicts,
		// and track registration timestamp and extension origin for debugging.

		Ok(())
	}

	async fn UnregisterCustomEditorProvider(&self, ViewType:String) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[CustomEditorProvider] Unregistering provider for view type: {}",
			ViewType
		);

		// Remove custom editor provider registration from ApplicationState. Should
		// check if any active editors are currently using this ViewType and either
		// force close with unsaved changes warning or prevent unregistration, remove
		// all stored configuration, capabilities, and sidecar association, notify the
		// sidecar extension to clean up its internal state, and remove any cached
		// resolution entries for this ViewType.

		Ok(())
	}

	async fn OnSaveCustomDocument(&self, ViewType:String, ResourceURI:Url) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[CustomEditorProvider] OnSaveCustomDocument called for '{}' at '{}'",
			ViewType,
			ResourceURI
		);

		// Workbench → extension save reverse-RPC. Cocoon's
		// `NotificationHandler.ts:781-810` already routes
		// `$onSaveCustomDocument` to the `customEditor.saveDocument`
		// emitter channel which fans out to whichever provider Cocoon's
		// `WindowNamespace.ts:188+` subscribed via `Subscribe(...)` at
		// `registerCustomEditorProvider` time. The extension's
		// `saveCustomDocument(document, cancellationToken)` callback
		// runs inside Cocoon - retrieves the edited content from the
		// webview, returns a `Thenable<void>` once the file has been
		// written. Mountain doesn't need to write the bytes itself; the
		// extension does that via its existing `vscode.workspace.fs`
		// shim which Cocoon already routes back into Mountain's
		// `FileSystem.WriteFile` IPC.
		//
		// Wire shape mirrors VS Code's
		// `vs/workbench/api/common/extHostCustom.ts::ExtHostCustomEditors`
		// `$onSaveCustomDocument` handler which expects positional args
		// `[CustomDocumentIdentifier, CancellationTokenId]`. Mountain
		// sends the resource URI as the document identifier (extension
		// stored the document under this key when it returned its
		// `CustomDocument` from `openCustomDocument`); the cancellation
		// token id is unused by our shim path and we send `0`.
		let IPCProvider:Arc<dyn IPCProvider> = self.Require();
		let DocumentIdentifier = json!({
			"viewType": ViewType,
			"resource": { "external": ResourceURI.to_string() },
		});
		let RPCMethod = format!("{}$onSaveCustomDocument", ProxyTarget::ExtHostCustomEditors.GetTargetPrefix());
		let RPCParameters = json!([DocumentIdentifier, 0]);
		match IPCProvider
			.SendRequestToSideCar("cocoon-main".to_string(), RPCMethod, RPCParameters, 30_000)
			.await
		{
			Ok(_) => {
				dev_log!(
					"extensions",
					"[CustomEditorProvider] OnSaveCustomDocument completed for '{}' at '{}'",
					ViewType,
					ResourceURI
				);
				let _ = self.ApplicationHandle.emit(
					"sky://customEditor/saved",
					json!({
						"viewType": ViewType,
						"resource": ResourceURI.to_string(),
					}),
				);
				Ok(())
			},
			Err(Error) => {
				dev_log!(
					"extensions",
					"warn: [CustomEditorProvider] OnSaveCustomDocument failed for '{}' at '{}': {:?}",
					ViewType,
					ResourceURI,
					Error
				);
				Err(Error)
			},
		}
	}

	async fn ResolveCustomEditor(
		&self,
		ViewType:String,
		ResourceURI:Url,
		WebviewPanelHandle:String,
	) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[CustomEditorProvider] Resolving custom editor for '{}' on resource '{}'",
			ViewType,
			ResourceURI
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
