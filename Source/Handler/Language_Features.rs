// ---------------------------------------------------------------------------------------------
// Mountain Language Features RPC Handlers 
// --------------------------------------------------------------------------------------------
// This module defines the `MainThreadLanguageFeaturesHandler` struct and its
// methods, which correspond to the `MainThreadLanguageFeaturesShape` RPC
// interface called by Cocoon (the extension host sidecar).
//
// Its primary responsibility is to handle the registration and unregistration
// of language feature providers (e.g., for hovers, completions, definitions)
// announced by Cocoon. It updates the central `AppState.language_providers`
// registry with this information.
//
// It also handles event notifications from Cocoon (e.g., when a provider's
// data changes).
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	ipc_effects::ProxyTarget, // For logging consistency if needed, not directly used for RPC method construction here
	language_feature_effects::{ProviderOptionsDto, ProviderType as CommonLangProviderType},
};
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, Wry}; // Wry for default runtime

use crate::{
	app_state::{AppState, ProviderRegistration},
	handlers::error_utils,
};

// Type alias for parameters received from Cocoon for registration calls.
// These are typically arrays like [handle, selectorDto, options?,
// extensionIdDto?].
type CocoonRpcParams = Value;
// Type alias for the IExtensionIdentifier DTO received from Cocoon.
type ExtensionIdDtoVal = Value; // Expected: { value: string, uuid?: string }

/// Handler for RPC calls from Cocoon related to language feature providers.
/// Instantiated by `track.rs` when routing sidecar requests.
#[derive(Clone)]
pub struct MainThreadLanguageFeaturesHandler {
	pub app_handle:AppHandle<Wry>,
}

impl MainThreadLanguageFeaturesHandler {
	// For providers with no specific options in their registration call (just
	// handle, selector, extId)
	macro_rules! impl_register_simple_provider {
	    ($method_name:ident, $provider_type:expr) => {
	        pub async fn $method_name(&self, sidecar_id: &str, params:
	CocoonRpcParams) -> Result<Value, String> {             let handle =
	params.get(0).and_then(Value::as_u64).map(|v| v as u32).ok_or_else(||
	error_utils::rpc_param_error_string(stringify!($method_name), "handle",
	"u32", Some(0)))?;             let selector_dto =
	params.get(1).cloned().ok_or_else(||
	error_utils::rpc_param_error_string(stringify!($method_name), "selectorDto",
	"array", Some(1)))?;             let extension_id_dto =
	params.get(2).cloned().ok_or_else(||
	error_utils::rpc_param_error_string(stringify!($method_name),
	"extensionIdDto", "object", Some(2)))?;             
	self.register_provider_in_app_state(sidecar_id, handle, $provider_type,
	selector_dto, None, extension_id_dto).await         }
	    };
	}

	impl_register_simple_provider!(registerDefinitionProvider, CommonLangProviderType::Definition);

	// extHost protocol might call $registerDefinitionSupport
	impl_register_simple_provider!(registerDeclarationProvider, CommonLangProviderType::Declaration);

	impl_register_simple_provider!(registerImplementationProvider, CommonLangProviderType::Implementation);

	impl_register_simple_provider!(registerTypeDefinitionProvider, CommonLangProviderType::TypeDefinition);

	impl_register_simple_provider!(registerDocumentHighlightProvider, CommonLangProviderType::DocumentHighlight);

	impl_register_simple_provider!(registerDocumentColorProvider, CommonLangProviderType::Color);

	impl_register_simple_provider!(registerReferenceProvider, CommonLangProviderType::References);

	// extHost protocol might call $registerReferenceSupport
	impl_register_simple_provider!(registerSelectionRangeProvider, CommonLangProviderType::SelectionRange);

	impl_register_simple_provider!(registerCallHierarchyProvider, CommonLangProviderType::CallHierarchy);

	impl_register_simple_provider!(registerTypeHierarchyProvider, CommonLangProviderType::TypeHierarchy);

	impl_register_simple_provider!(registerLinkedEditingRangeProvider, CommonLangProviderType::LinkedEditingRange);

	// For SemanticTokens, extHost.protocol registers
	// $registerDocumentSemanticTokensProvider and
	// $registerDocumentRangeSemanticTokensProvider These take a legend as options.

	// For providers that take an options DTO (ProviderOptionsDto) as params[2]
	macro_rules! impl_register_provider_with_options {
	    ($method_name:ident, $provider_type:expr) => {
	        pub async fn $method_name(&self, sidecar_id: &str, params:
	CocoonRpcParams) -> Result<Value, String> {             let handle =
	params.get(0).and_then(Value::as_u64).map(|v| v as u32).ok_or_else(||
	error_utils::rpc_param_error_string(stringify!($method_name), "handle",
	"u32", Some(0)))?;             let selector_dto =
	params.get(1).cloned().ok_or_else(||
	error_utils::rpc_param_error_string(stringify!($method_name), "selectorDto",
	"array", Some(1)))?;             let options_val = params.get(2).cloned();
	// This is ProviderOptionsDto as Value             let extension_id_dto =
	params.get(3).cloned().ok_or_else(||
	error_utils::rpc_param_error_string(stringify!($method_name),
	"extensionIdDto", "object", Some(3)))?;             let options_dto:
	Option<ProviderOptionsDto> = options_val.and_then(|v| if v.is_null() { None }
	else { serde_json::from_value(v.clone()).map_err(|e| warn!("[MtnLangFeat RPC]
	Failed deserialize ProviderOptionsDto for {}: {}. Val: {:?}",
	stringify!($method_name), e, v)).ok() });             
	self.register_provider_in_app_state(sidecar_id, handle, $provider_type,
	selector_dto, options_dto, extension_id_dto).await         }
	    };
	}

	impl_register_provider_with_options!(registerCodeLensProvider, CommonLangProviderType::CodeLens);

	// $registerCodeLensSupport
	impl_register_provider_with_options!(registerCodeActionProvider, CommonLangProviderType::CodeAction);

	// $registerCodeActionSupport
	impl_register_provider_with_options!(registerDocumentFormattingEditProvider, CommonLangProviderType::Formatting);

	// $registerDocumentFormattingSupport
	impl_register_provider_with_options!(
		registerDocumentRangeFormattingEditProvider,
		CommonLangProviderType::RangeFormatting
	);

	// $registerDocumentRangeFormattingSupport
	impl_register_provider_with_options!(
		registerOnTypeFormattingEditProvider,
		CommonLangProviderType::OnTypeFormatting
	);

	// $registerOnTypeFormattingSupport
	impl_register_provider_with_options!(registerDocumentLinkProvider, CommonLangProviderType::DocumentLink);

	impl_register_provider_with_options!(registerFoldingRangeProvider, CommonLangProviderType::FoldingRange);

	impl_register_provider_with_options!(registerRenameProvider, CommonLangProviderType::Rename);

	// $registerRenameSupport
	impl_register_provider_with_options!(registerSignatureHelpProvider, CommonLangProviderType::SignatureHelp);

	impl_register_provider_with_options!(registerWorkspaceSymbolProvider, CommonLangProviderType::WorkspaceSymbol);

	// Selector is null for this usually
	impl_register_provider_with_options!(registerDocumentSymbolProvider, CommonLangProviderType::DocumentSymbol);

	impl_register_provider_with_options!(registerInlayHintsProvider, CommonLangProviderType::InlayHints);

	pub fn new(app_handle:AppHandle<Wry>) -> Self { Self { app_handle } }

	/// Internal helper to register a provider in `AppState`.
	async fn register_provider_in_app_state(
		&self,
		sidecar_id:&str,
		handle_from_cocoon:u32, // Handle generated by Cocoon for its internal tracking
		provider_type:CommonLangProviderType,
		selector_dto:Value,                     // JSON Value for IDocumentFilterDto[]
		options_dto:Option<ProviderOptionsDto>, // Parsed specific options DTO
		extension_id_dto:ExtensionIdDtoVal,     // JSON Value for IExtensionIdentifierDto
	) -> Result<Value, String> {
		let extension_id_str = extension_id_dto
			.get("value")
			.and_then(Value::as_str)
			.unwrap_or("unknown_extension");

		info!(
			"[MtnLangFeat RPC] Register {:?}Provider: CocoonHandle={}, MountainHandle will be new, Ext='{}', \
			 Sid='{}', OptsIsSome={}",
			provider_type,
			handle_from_cocoon,
			extension_id_str,
			sidecar_id,
			options_dto.is_some()
		);
		trace!(
			"[MtnLangFeat RPC] Register {:?}Provider Details: Selector='{:?}', Options='{:?}', ExtID='{:?}'",
			provider_type, selector_dto, options_dto, extension_id_dto
		);

		let app_state = self.app_handle.state::<AppState>();
		// Mountain generates its own handle for AppState storage.
		// The handle_from_cocoon is what Cocoon uses to identify its provider instance.
		// When Mountain calls back to Cocoon (e.g., $provideHover), it uses
		// handle_from_cocoon.
		let mountain_provider_handle = app_state.get_next_provider_handle();

		let mut providers_map_guard = app_state
			.language_providers
			.lock()
			.map_err(|e| error_utils::format_app_state_lock_error("language_providers for register", e))?;

		// We store using Mountain's handle. If Cocoon's handle needs to be findable,
		// ProviderRegistration might need to store it, or a separate map could exist.
		// For now, assume Mountain's handle is the primary key.
		// The `handle` field in `ProviderRegistration` will be
		// `mountain_provider_handle`. The `handle_from_cocoon` is used by Cocoon, so
		// we need to store it too if we want to reference it later for specific
		// provider calls. Let's assume the handle in ProviderRegistration
		// IS the handle Cocoon sent, making it easier to call back.

		// Decision: For simplicity and direct mapping to Cocoon's calls, let's use
		// Cocoon's handle as the key in Mountain's AppState.language_providers. This
		// means Mountain's `next_provider_handle` might not be used for these, or
		// it's used for providers Mountain itself creates. Let's stick to the model
		// where track.rs calls this with `handle_from_cocoon` and that's the key.

		if providers_map_guard.contains_key(&handle_from_cocoon) {
			warn!(
				"[MtnLangFeat RPC] Provider with Cocoon handle {} is already registered. Overwriting. (Type: {:?}, \
				 Ext: '{}')",
				handle_from_cocoon, provider_type, extension_id_str
			);
		}

		providers_map_guard.insert(
			handle_from_cocoon, // Use Cocoon's handle as the key
			ProviderRegistration {
				handle:handle_from_cocoon, // Store Cocoon's handle
				provider_type,
				selector:selector_dto,
				sidecar_id:sidecar_id.to_string(),
				options:options_dto,
				extension_id:extension_id_dto,
			},
		);
		debug!(
			"[MtnLangFeat RPC] Provider for Ext '{}', Type {:?}, Handle {} registered successfully.",
			extension_id_str, provider_type, handle_from_cocoon
		);
		Ok(Value::Null) // Registration RPCs are typically void or return a simple ack.
	}

	// --- Implementations for MainThreadLanguageFeaturesShape $register... methods
	// --- Parameters must match what Cocoon's ShimLanguageFeatures sends.
	// Cocoon typically sends: [handle: u32, selectorDto: Value,
	// optionsSpecificToProvider?: Value, extensionIdDto: Value]
	// The 'optionsSpecificToProvider' needs to be parsed into `ProviderOptionsDto`.

	pub async fn registerHoverProvider(&self, sidecar_id:&str, params:CocoonRpcParams) -> Result<Value, String> {
		let handle =
			params.get(0).and_then(Value::as_u64).map(|v| v as u32).ok_or_else(|| {
				error_utils::rpc_param_error_string("registerHoverProvider", "handle", "u32", Some(0))
			})?;
		let selector_dto = params.get(1).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("registerHoverProvider", "selectorDto", "array", Some(1))
		})?;
		let extension_id_dto = params.get(2).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("registerHoverProvider", "extensionIdDto", "object", Some(2))
		})?; // Hover has no options in base protocol
		self.register_provider_in_app_state(
			sidecar_id,
			handle,
			CommonLangProviderType::Hover,
			selector_dto,
			None,
			extension_id_dto,
		)
		.await
	}

	pub async fn registerCompletionItemProvider(
		&self,
		sidecar_id:&str,
		params:CocoonRpcParams,
	) -> Result<Value, String> {
		// Params: [handle, selector, optionsDto (for Completion), extensionIdDto]
		let handle = params.get(0).and_then(Value::as_u64).map(|v| v as u32).ok_or_else(|| {
			error_utils::rpc_param_error_string("registerCompletionProvider", "handle", "u32", Some(0))
		})?;
		let selector_dto = params.get(1).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("registerCompletionProvider", "selectorDto", "array", Some(1))
		})?;
		let options_val = params.get(2).cloned(); // This is the ProviderOptionsDto for completion
		let extension_id_dto = params.get(3).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("registerCompletionProvider", "extensionIdDto", "object", Some(3))
		})?;
		let options_dto = options_val.and_then(|v| if v.is_null() { None } else { serde_json::from_value(v).ok() });
		self.register_provider_in_app_state(
			sidecar_id,
			handle,
			CommonLangProviderType::Completion,
			selector_dto,
			options_dto,
			extension_id_dto,
		)
		.await
	}

	// TODO: Semantic Tokens providers will take SemanticTokensLegendDto within
	// ProviderOptionsDto.

	pub async fn unregisterProvider(&self, sidecar_id:&str, params:CocoonRpcParams) -> Result<Value, String> {
		let handle = params
			.get(0)
			.and_then(Value::as_u64)
			.map(|v| v as u32)
			.ok_or_else(|| error_utils::rpc_param_error_string("unregisterProvider", "handle", "u32", Some(0)))?;
		info!(
			"[MtnLangFeat RPC] Unregistering provider: Handle={}, Sidecar='{}'",
			handle, sidecar_id
		);
		let app_state = self.app_handle.state::<AppState>();
		let mut providers_map_guard = app_state
			.language_providers
			.lock()
			.map_err(|e| error_utils::format_app_state_lock_error("language_providers for unregister", e))?;
		if let Some(removed_entry) = providers_map_guard.remove(&handle) {
			if removed_entry.sidecar_id != sidecar_id {
				warn!(
					"[MtnLangFeat RPC] Sidecar '{}' unregistered provider {} owned by '{}'.",
					sidecar_id, handle, removed_entry.sidecar_id
				);
			}
			debug!(
				"[MtnLangFeat RPC] Provider handle {} (Type: {:?}) unregistered.",
				handle, removed_entry.provider_type
			);
		} else {
			warn!("[MtnLangFeat RPC] Attempted unregister non-existent handle: {}", handle);
		}
		Ok(Value::Null)
	}

	// --- Event Emitters from Cocoon ---
	pub async fn emitCodeLensEvent(&self, _sidecar_id:&str, params:CocoonRpcParams) -> Result<Value, String> {
		let event_handle =
			params.get(0).and_then(Value::as_u64).map(|v| v as u32).ok_or_else(|| {
				error_utils::rpc_param_error_string("emitCodeLensEvent", "eventHandle", "u32", Some(0))
			})?;
		info!(
			"[MtnLangFeat RPC] Received onDidChangeCodeLenses event for handle: {}",
			event_handle
		);
		// TODO: Invalidate CodeLens cache for this event_handle.
		Ok(Value::Null)
	}
	// TODO: Implement $emitInlayHintsEvent, $emitFoldingRangeEvent,
	// $emitDocumentSemanticTokensEvent etc.
}
