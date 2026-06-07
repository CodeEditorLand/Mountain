//! # CustomEditorProvider (Environment)
//!
//! Implements
//! [`CustomEditorProvider`](CommonLibrary::CustomEditor::CustomEditorProvider)
//! for `MountainEnvironment`, managing registration and lifecycle of custom
//! non-text editors. Coordinates Webview-based editing experiences (SVG
//! editors, diff viewers, etc.) and handles editor resolution, save
//! operations, and provider unregistration.
//!
//! Uses [`IPCProvider`](CommonLibrary::IPC::IPCProvider) for RPC communication
//! with Cocoon and integrates with `ApplicationState` for provider registration
//! persistence.
//!
//! ## Methods
//!
//! - `RegisterCustomEditorProvider` - register extension provider by view type
//! - `UnregisterCustomEditorProvider` - unregister provider
//! - `OnSaveCustomDocument` - workbench → extension save reverse-RPC via
//!   `$onSaveCustomDocument`; returns the sidecar's error verbatim on failure
//! - `ResolveCustomEditor` - fire-and-forget RPC to populate the webview
//!
//! ## VS Code reference
//!
//! - `vs/workbench/contrib/customEditor/browser/customEditorService.ts`
//! - `vs/workbench/contrib/customEditor/common/customEditor.ts`

use std::{
	collections::HashMap,
	sync::{Arc, Mutex, OnceLock},
};

use CommonLibrary::{
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::{Emitter, Manager};
use url::Url;

use super::MountainEnvironment::MountainEnvironment;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::CreateEffectForRequest::Utilities::Proxy::proxy_cocoon,
	dev_log,
};

/// Process-global custom editor registry: ViewType → SidecarId.
/// Populated by `RegisterCustomEditorProvider`, consumed by
/// `ResolveCustomEditor` to route the save/resolve RPC to the
/// correct extension host sidecar.
static CUSTOM_EDITOR_REGISTRY:OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn GetRegistry() -> &'static Mutex<HashMap<String, String>> {
	CUSTOM_EDITOR_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Return the sidecar identifier for a registered ViewType, or
/// `"cocoon-main"` as the canonical fallback.
pub fn LookupSidecarForViewType(ViewType:&str) -> String {
	GetRegistry()
		.lock()
		.ok()
		.and_then(|G| G.get(ViewType).cloned())
		.unwrap_or_else(|| "cocoon-main".to_string())
}

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

		// Store ViewType → "cocoon-main" (the only extension host sidecar
		// today). When Grove multi-extension-host lands, the sidecar id will
		// come from the Options payload.
		let SidecarId = _Options
			.get("sidecarId")
			.and_then(Value::as_str)
			.unwrap_or("cocoon-main")
			.to_string();

		if let Ok(mut Registry) = GetRegistry().lock() {
			let IsNew = !Registry.contains_key(&ViewType);

			Registry.insert(ViewType.clone(), SidecarId.clone());

			dev_log!(
				"extensions",
				"[CustomEditorProvider] {} provider registered: viewType={} sidecar={}",
				if IsNew { "New" } else { "Updated" },
				ViewType,
				SidecarId
			);
		}

		Ok(())
	}

	async fn UnregisterCustomEditorProvider(&self, ViewType:String) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[CustomEditorProvider] Unregistering provider for view type: {}",
			ViewType
		);

		if let Ok(mut Registry) = GetRegistry().lock() {
			let Removed = Registry.remove(&ViewType).is_some();

			dev_log!(
				"extensions",
				"[CustomEditorProvider] Provider unregistered: viewType={} (was_present={})",
				ViewType,
				Removed
			);
		}

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
		let run_time:Arc<ApplicationRunTime> =
			self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		let DocumentIdentifier = json!({
			"viewType": ViewType,
			"resource": { "external": ResourceURI.to_string() },
		});

		let RPCParameters = json!([DocumentIdentifier, 0]);

		match proxy_cocoon(
			&run_time,
			ProxyTarget::ExtHostCustomEditors,
			"onSaveCustomDocument",
			RPCParameters,
			30_000,
		)
		.await
		.map_err(|e| CommonError::IPCError { Description:e })
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
