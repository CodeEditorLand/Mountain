// File: Ipc/Rpc/ServiceIdentifierMap.rs
// Defines a static map that translates string-based service identifiers from
// RPC calls to the corresponding `ExtHostContext` enum variants used internally
// by VS Code's RPCProtocol.

#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;

use once_cell::sync::Lazy;
use vs_workbench_api_common_exthost_protocol::ExtHostContext;

/// A lazy-initialized static HashMap to map service name strings to their
/// `ExtHostContext` enum. This is crucial for the RPC dispatcher to find the
/// correct local service instance (shim) to handle an incoming call from
/// Mountain.
pub static SERVICE_ID_TO_EXT_HOST_CONTEXT_MAP:Lazy<HashMap<String, ExtHostContext>> = Lazy::new(|| {
	let mut MapInstance = HashMap::new();
	MapInstance.insert("ExtHostConfiguration".to_string(), ExtHostContext::ExtHostConfiguration);
	MapInstance.insert("ExtHostDocuments".to_string(), ExtHostContext::ExtHostDocuments);
	MapInstance.insert("ExtHostWorkspace".to_string(), ExtHostContext::ExtHostWorkspace);
	MapInstance.insert("ExtHostDiagnostics".to_string(), ExtHostContext::ExtHostDiagnostics);
	MapInstance.insert("ExtHostLanguageFeatures".to_string(), ExtHostContext::ExtHostLanguageFeatures);
	MapInstance.insert("ExtHostOutputService".to_string(), ExtHostContext::ExtHostOutputService);
	MapInstance.insert("ExtHostTerminalService".to_string(), ExtHostContext::ExtHostTerminalService);
	MapInstance.insert("ExtHostStorage".to_string(), ExtHostContext::ExtHostStorage);
	MapInstance.insert("ExtHostTelemetry".to_string(), ExtHostContext::ExtHostTelemetry);
	MapInstance.insert("ExtHostEnv".to_string(), ExtHostContext::ExtHostEnv);
	MapInstance.insert(
		"ExtHostExtensionEnablement".to_string(),
		ExtHostContext::ExtHostExtensionEnablement,
	);
	MapInstance.insert("ExtHostFileSystemInfo".to_string(), ExtHostContext::ExtHostFileSystemInfo);
	MapInstance.insert("ExtHostDebugService".to_string(), ExtHostContext::ExtHostDebugService);
	MapInstance.insert("ExtHostTaskService".to_string(), ExtHostContext::ExtHostTaskService);
	MapInstance.insert("ExtHostAuthentication".to_string(), ExtHostContext::ExtHostAuthentication);
	MapInstance.insert("ExtHostChatProvider".to_string(), ExtHostContext::ExtHostChatProvider);
	MapInstance.insert("ExtHostWebviews".to_string(), ExtHostContext::ExtHostWebviews);
	MapInstance.insert("ExtHostWebviewPanels".to_string(), ExtHostContext::ExtHostWebviewPanels);
	MapInstance.insert("ExtHostWebviewViews".to_string(), ExtHostContext::ExtHostWebviewViews);
	MapInstance.insert("ExtHostCustomEditors".to_string(), ExtHostContext::ExtHostCustomEditors);
	MapInstance.insert("ExtHostQuickInput".to_string(), ExtHostContext::ExtHostQuickInput);
	MapInstance.insert("ExtHostMessageService".to_string(), ExtHostContext::ExtHostMessageService);
	// Add other services as they are shimmed and registered.
	MapInstance
});
