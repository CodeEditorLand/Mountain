// ---------------------------------------------------------------------------------------------
// Mountain Environment - Language Features Provider Registry
// 
// --------------------------------------------------------------------------------------------
// This module implements the `LanguageFeatureProviderRegistry` trait for
// `MountainEnvironment`. It manages the registration of language feature
// providers from sidecars (e.g., Cocoon) and handles the invocation of these
// features by making RPC calls to the appropriate sidecar.
//
// Key Responsibilities:
// - Registering and unregistering providers in `AppState`.
// - Finding active providers for a given document/context using
//   `DocumentSelector` matching.
// - For each language feature (hover, completion, definition, etc.):
//   - Constructing RPC parameters.
//   - Making an RPC call to the sidecar (via `IpcProvider` / `vine`).
//   - Deserializing the response from the sidecar into common DTOs.
// - Managing context for stateful operations like hierarchy requests.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	environment::Requires,
	errors::CommonError,
	ipc_effects::{IpcProvider, ProxyTarget}, // For constructing RPC method names
	language_feature_effects::{
		CodeActionContextDto,
		CodeActionDto,
		CodeActionListDto,
		CodeLensDto,
		CodeLensListDto,
		CompletionContextDto,
		DocumentHighlightDto,
		DocumentSymbolDto,
		FoldingRangeDto,
		FormattingOptionsDto,
		HierarchyItemDto,
		HoverResultDto,
		IncomingCallDto,
		InlayHintDto,
		LanguageFeatureProviderRegistry,
		LinkDto,
		LinkedEditingRangesDto,
		LinksListDto,
		LocationLinkDto,
		OutgoingCallDto,
		PositionDto,
		ProviderDescription,
		ProviderOptionsDto,
		ProviderType as CommonProviderType,
		RangeDto,
		SelectionRangeDto,
		SemanticTokensDto,
		SemanticTokensEditsDto,
		SignatureHelpContextDto,
		SignatureHelpResultDto,
		SuggestResultDto,
		TextEditDto,
		WorkspaceEditDto,
		WorkspaceSymbolDto,
	},
};
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use url::Url;

use crate::{
	app_state::{AppState, HierarchySessionContext, ProviderRegistration}, /* For storing registrations and hierarchy
	                                                                       * sessions */
	environment::{MountainEnvironment, utils::map_app_state_lock_error_to_common_error},
	handlers, /* For config::match_document_selector
	           * vine, // Accessed via IpcProvider */
};

// --- LanguageFeatureProviderRegistry Implementation ---
#[async_trait]
impl LanguageFeatureProviderRegistry for MountainEnvironment {
	async fn register_provider(
		&self,
		sidecar_id:String,
		provider_type:CommonProviderType,
		selector_dto:Value,                     // DocumentSelector DTO
		extension_id_dto:Value,                 // IExtensionIdentifier DTO
		options_dto:Option<ProviderOptionsDto>, // ProviderOptionsDto
	) -> Result<u32, CommonError> {
		let app_state = self.get_app_state();
		let new_provider_handle = app_state.get_next_provider_handle();

		info!(
			"[Env LangFeatReg] Register: Type='{:?}', Handle={}, SidecarID='{}', ExtID='{:?}', OptionsIsSome={}",
			provider_type,
			new_provider_handle,
			sidecar_id,
			extension_id_dto.get("value"),
			options_dto.is_some()
		);
		trace!(
			"[Env LangFeatReg Register] Selector: {:?}, Options: {:?}, ExtensionID: {:?}",
			selector_dto, options_dto, extension_id_dto
		);

		let new_registration = ProviderRegistration {
			handle:new_provider_handle,
			provider_type, // Directly use CommonProviderType
			selector:selector_dto,
			sidecar_id,
			extension_id:extension_id_dto,
			options:options_dto,
		};

		app_state
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.insert(new_provider_handle, new_registration);

		Ok(new_provider_handle)
	}

	async fn unregister_provider(&self, provider_handle_to_remove:u32) -> Result<(), CommonError> {
		info!("[Env LangFeatReg] Unregister: Handle={}", provider_handle_to_remove);
		if self
			.get_app_state()
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&provider_handle_to_remove)
			.is_none()
		{
			warn!(
				"[Env LangFeatReg Unregister] Attempted to unregister non-existent handle: {}",
				provider_handle_to_remove
			);
		}
		Ok(())
	}

	// This is an internal helper, not part of the public trait, but essential for
	// provider invocation. It could also live in `utils.rs` if preferred.
	async fn get_providers_for_document_internal(
		&self,
		document_uri:&Url,
		language_id:&str,
		provider_type:CommonProviderType,
	) -> Result<Vec<ProviderRegistration>, CommonError> {
		// Returns Vec<ProviderRegistration>
		debug!(
			"[Env LangFeatReg GetInternal] For Doc='{}', Lang='{}', Type='{:?}'",
			document_uri.path_segments().and_then(|s| s.last()).unwrap_or_default(),
			language_id,
			provider_type
		);
		let app_state_val = self.get_app_state();
		let providers_map_guard = app_state_val
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		let matching_providers:Vec<ProviderRegistration> = providers_map_guard.values()
            .filter(|reg| reg.provider_type == provider_type)
            .filter(|reg| handlers::config::match_document_selector(®.selector, document_uri, language_id))
            .cloned() // Clone the ProviderRegistration data
            .collect();

		debug!(
			"[Env LangFeatReg GetInternal] Found {} matching {:?} providers for doc='{}', lang='{}'",
			matching_providers.len(),
			provider_type,
			document_uri.path_segments().and_then(|s| s.last()).unwrap_or_default(),
			language_id
		);
		Ok(matching_providers)
	}

	// --- Provider Invocation Methods ---

	// // Helper macro to reduce boilerplate for simple "provide" methods
	macro_rules! provide_feature {
	    ($self:ident, $method_rpc_name:literal, $doc_uri:expr, $lang_id:expr,
	$provider_type:expr, $params_fn:expr, $result_dto:ty, $timeout:expr) => {{
	        let providers = $self.get_providers_for_document_internal(&$doc_uri,
	&$lang_id, $provider_type).await?;         if let Some(provider_reg) =
	providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) { // MVP: find
	first Cocoon provider             info!("[Env LangFeat {}] Calling Cocoon
	provider (H:{}) for {}", $method_rpc_name, provider_reg.handle, $doc_uri);
	            let rpc_params_array = $params_fn(provider_reg.handle,
	&$doc_uri);             let rpc_method = format!("{}${}",
	ProxyTarget::ExtHostLanguageFeatures.target_prefix(), $method_rpc_name);
	            let ipc_provider: Arc<dyn IpcProvider + Send + Sync> =
	$self.require();             match
	ipc_provider.send_request_to_sidecar(provider_reg.sidecar_id.clone(),
	rpc_method, Value::Array(rpc_params_array), $timeout).await {               
	Ok(v) if !v.is_null() => serde_json::from_value(v).map_err(|e|
	CommonError::IpcError(format!("Deserialize {}: {}", stringify!($result_dto),
	e))).map(Some),                 Ok(_) => Ok(None),
	                Err(e) => Err(CommonError::IpcError(format!("RPC for {}
	failed: {}", $method_rpc_name, e))),             }
	        } else {
	            debug!("[Env LangFeat {}] No Cocoon provider found for {}",
	$method_rpc_name, $doc_uri);             Ok(None)
	        }
	    }};
	}
	// Helper macro for "resolve" methods
	macro_rules! resolve_feature_item {
	    ($self:ident, $method_rpc_name:literal, $list_cache_id:expr,
	$item_dto_val:expr, $token_val:expr, $provider_type:expr, $result_dto:ty,
	$timeout:expr) => {{         // Use list_cache_id (provider handle) to get
	sidecar_id         let provider_reg =
	$self.get_provider_registration_from_handle($list_cache_id,
	$provider_type).await?;         info!("[Env LangFeat {}] Calling Cocoon
	provider (H:{}, SID:{}) for resolve", $method_rpc_name, provider_reg.handle,
	provider_reg.sidecar_id);         let rpc_params =
	json!([provider_reg.handle, $item_dto_val,
	$token_val.unwrap_or(Value::Null)]);         let rpc_method =
	format!("{}${}", ProxyTarget::ExtHostLanguageFeatures.target_prefix(),
	$method_rpc_name);         let ipc_provider: Arc<dyn IpcProvider + Send +
	Sync> = $self.require();         match
	ipc_provider.send_request_to_sidecar(provider_reg.sidecar_id.clone(),
	rpc_method, rpc_params, $timeout).await {             Ok(v) if !v.is_null()
	=> serde_json::from_value(v).map_err(|e|
	CommonError::IpcError(format!("Deserialize {}: {}", stringify!($result_dto),
	e))).map(Some),             Ok(_) => Ok(None),
	            Err(e) => Err(CommonError::IpcError(format!("RPC for {} failed:
	{}", $method_rpc_name, e))),         }
	    }};
	}

	async fn provide_hover(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto, // , token_val: Option<Value> - implicit in effect constructor
	) -> Result<Option<HoverResultDto>, CommonError> {
		provide_feature!(
			self,
			"provideHover",
			document_uri,
			language_id,
			CommonProviderType::Hover,
			|handle, uri_ref, pos_dto_val:PositionDto, ctx_val:Value, token_val:Value| {
				// Renamed to match macro params
				vec![
					json!(handle),
					json!({"scheme": uri_ref.scheme(), "path": uri_ref.path(), "external": uri_ref.to_string(), "$mid": 1}),
					json!(pos_dto_val),
					ctx_val,
					token_val,
				]
			},
			HoverResultDto,
			5000 // timeout
		)
	}

	async fn provide_completions(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		context_dto:CompletionContextDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<SuggestResultDto>, CommonError> {
		provide_feature!(
			self,
			"provideCompletionItems",
			document_uri,
			language_id,
			CommonProviderType::Completion,
			|handle, uri_ref, pos_dto_val:PositionDto, ctx_dto_val:CompletionContextDto, token_val:Value| {
				vec![
					json!(handle),
					json!({"scheme": uri_ref.scheme(), "path": uri_ref.path(), "external": uri_ref.to_string(), "$mid": 1}),
					json!(pos_dto_val),
					json!(ctx_dto_val),
					token_val,
				]
			},
			SuggestResultDto,
			5000
		)
	}

	async fn resolve_completion_item_for_list(
		&self,
		list_cache_id:u32,
		item_to_resolve_dto:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		// Resolve methods return Value directly as the DTO can be complex or vary.
		let provider_reg = self
			.get_provider_registration_from_handle(list_cache_id, CommonProviderType::Completion)
			.await?;
		info!(
			"[Env LangFeat CompResolve] Calling Cocoon provider (H:{}, SID:{}) for resolve",
			provider_reg.handle, provider_reg.sidecar_id
		);
		let rpc_params = json!([
			provider_reg.handle,
			item_to_resolve_dto,
			cancellation_token_id_val.unwrap_or(Value::Null)
		]);
		let rpc_method = format!("{}$resolveCompletionItem", ProxyTarget::ExtHostLanguageFeatures.target_prefix());
		let ipc_provider:Arc<dyn IpcProvider + Send + Sync> = self.require();
		match ipc_provider
			.send_request_to_sidecar(provider_reg.sidecar_id.clone(), rpc_method, rpc_params, 5000)
			.await
		{
			Ok(v) if !v.is_null() => Ok(Some(v)),
			Ok(_) => Ok(None),
			Err(e) => Err(CommonError::IpcError(format!("RPC for resolveCompletionItem: {}", e))),
		}
	}

	async fn provide_definition(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		provide_feature!(
			self,
			"provideDefinition",
			document_uri,
			language_id,
			CommonProviderType::Definition,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token], /* Context not used by
			                                                                          * $provideDefinition */
			Vec<LocationLinkDto>,
			5000
		)
	}

	async fn provide_declaration(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		provide_feature!(
			self,
			"provideDeclaration",
			document_uri,
			language_id,
			CommonProviderType::Declaration,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			Vec<LocationLinkDto>,
			5000
		)
	}

	async fn provide_implementation(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		provide_feature!(
			self,
			"provideImplementation",
			document_uri,
			language_id,
			CommonProviderType::Implementation,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			Vec<LocationLinkDto>,
			5000
		)
	}

	async fn provide_type_definition(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		provide_feature!(
			self,
			"provideTypeDefinition",
			document_uri,
			language_id,
			CommonProviderType::TypeDefinition,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			Vec<LocationLinkDto>,
			5000
		)
	}

	async fn provide_code_actions(
		&self,
		document_uri:Url,
		language_id:String,
		range_or_selection_dto:Value,
		context_dto:CodeActionContextDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<CodeActionListDto>, CommonError> {
		provide_feature!(
			self,
			"provideCodeActions",
			document_uri,
			language_id,
			CommonProviderType::CodeAction,
			|h, uri, range_sel_val:Value, ctx_dto_val:CodeActionContextDto, token_val:Value| {
				// Note: range_sel_val comes from macro
				vec![json!(h), uri, range_sel_val, json!(ctx_dto_val), token_val]
			},
			CodeActionListDto,
			5000
		)
	}

	async fn resolve_code_action(
		&self,
		list_cache_id:u32,
		_ignored_sidecar_id:String,
		action_to_resolve_dto:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<CodeActionDto>, CommonError> {
		resolve_feature_item!(
			self,
			"resolveCodeAction",
			list_cache_id,
			action_to_resolve_dto,
			cancellation_token_id_val,
			CommonProviderType::CodeAction,
			CodeActionDto,
			5000
		)
	}

	async fn provide_code_lenses(
		&self,
		document_uri:Url,
		language_id:String,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<CodeLensListDto>, CommonError> {
		provide_feature!(
			self,
			"provideCodeLenses",
			document_uri,
			language_id,
			CommonProviderType::CodeLens,
			|h, uri, _pos_ignored:Value, _ctx_ignored:Value, token| vec![json!(h), uri, token], /* provideCodeLenses
			                                                                                     * doesn't take
			                                                                                     * position/context */
			CodeLensListDto,
			5000
		)
	}

	async fn resolve_code_lens(
		&self,
		list_cache_id:u32,
		_ignored_sidecar_id:String,
		lens_to_resolve_dto:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<CodeLensDto>, CommonError> {
		resolve_feature_item!(
			self,
			"resolveCodeLens",
			list_cache_id,
			lens_to_resolve_dto,
			cancellation_token_id_val,
			CommonProviderType::CodeLens,
			CodeLensDto,
			5000
		)
	}

	async fn provide_document_symbols(
		&self,
		document_uri:Url,
		language_id:String,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<DocumentSymbolDto>>, CommonError> {
		provide_feature!(
			self,
			"provideDocumentSymbols",
			document_uri,
			language_id,
			CommonProviderType::DocumentSymbol,
			|h, uri, _pos:Value, _ctx:Value, token| vec![json!(h), uri, token],
			Vec<DocumentSymbolDto>,
			5000
		)
	}

	async fn provide_workspace_symbols(
		&self,
		query:String,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<WorkspaceSymbolDto>>, CommonError> {
		// Workspace symbols are not document-specific. We need to find any Cocoon
		// provider for this.
		let app_state = self.get_app_state();
		let providers_guard = app_state
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;
		if let Some(p_reg) = providers_guard
			.values()
			.find(|reg| {
				reg.provider_type == CommonProviderType::WorkspaceSymbol && reg.sidecar_id.starts_with("cocoon")
			})
			.cloned()
		{
			drop(providers_guard);
			info!(
				"[Env LangFeat WSSymbols] Calling Cocoon provider (H:{}) for query '{}'",
				p_reg.handle, query
			);
			let rpc_params = json!([p_reg.handle, query, cancellation_token_id_val.unwrap_or(Value::Null)]);
			let rpc_method = format!(
				"{}$provideWorkspaceSymbols",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc_provider:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc_provider
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 15000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<WorkspaceSymbolDto>: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for workspace symbols: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_signature_help(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		context_dto:SignatureHelpContextDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<SignatureHelpResultDto>, CommonError> {
		provide_feature!(
			self,
			"provideSignatureHelp",
			document_uri,
			language_id,
			CommonProviderType::SignatureHelp, // Added SignatureHelp to CommonProviderType
			|h, uri, pos, ctx:SignatureHelpContextDto, token| vec![json!(h), uri, json!(pos), json!(ctx), token],
			SignatureHelpResultDto,
			5000
		)
	}

	async fn provide_document_formatting_edits(
		&self,
		document_uri:Url,
		language_id:String,
		options_dto:FormattingOptionsDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<TextEditDto>>, CommonError> {
		provide_feature!(
			self,
			"provideDocumentFormattingEdits",
			document_uri,
			language_id,
			CommonProviderType::Formatting,
			|h, uri, _pos_ignored:Value, opts:FormattingOptionsDto, token| vec![json!(h), uri, json!(opts), token], /* Formatting takes options, not position */
			Vec<TextEditDto>,
			10000
		)
	}

	async fn provide_document_range_formatting_edits(
		&self,
		document_uri:Url,
		language_id:String,
		range_dto:RangeDto,
		options_dto:FormattingOptionsDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<TextEditDto>>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::RangeFormatting)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(range_dto),
				json!(options_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!(
				"{}$provideDocumentRangeFormattingEdits",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 10000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<TextEditDto>: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for range formatting: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_on_type_formatting_edits(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		ch:String,
		options_dto:FormattingOptionsDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<TextEditDto>>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::OnTypeFormatting)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(position_dto),
				ch,
				json!(options_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!(
				"{}$provideOnTypeFormattingEdits",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 5000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<TextEditDto>: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for on-type formatting: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_document_highlights(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<DocumentHighlightDto>>, CommonError> {
		provide_feature!(
			self,
			"provideDocumentHighlights",
			document_uri,
			language_id,
			CommonProviderType::DocumentHighlight,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			Vec<DocumentHighlightDto>,
			5000
		)
	}

	async fn provide_document_links(
		&self,
		document_uri:Url,
		language_id:String,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<LinksListDto>, CommonError> {
		provide_feature!(
			self,
			"provideDocumentLinks",
			document_uri,
			language_id,
			CommonProviderType::DocumentLink,
			|h, uri, _pos:Value, _ctx:Value, token| vec![json!(h), uri, token],
			LinksListDto,
			5000
		)
	}

	async fn resolve_document_link(
		&self,
		list_cache_id:u32,
		_ignored_sidecar_id:String,
		link_dto_val:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<LinkDto>, CommonError> {
		resolve_feature_item!(
			self,
			"resolveDocumentLink",
			list_cache_id,
			link_dto_val,
			cancellation_token_id_val,
			CommonProviderType::DocumentLink,
			LinkDto,
			5000
		)
	}

	async fn provide_references(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		context_dto:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		provide_feature!(
			self,
			"provideReferences",
			document_uri,
			language_id,
			CommonProviderType::References,
			|h, uri, pos, ctx:Value, token| vec![json!(h), uri, json!(pos), ctx, token],
			Vec<LocationLinkDto>,
			15000
		)
	}

	async fn prepare_rename(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::Rename)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(position_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!("{}$prepareRename", ProxyTarget::ExtHostLanguageFeatures.target_prefix());
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 5000)
				.await
			{
				Ok(v) if !v.is_null() => Ok(Some(v)),
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for prepareRename: {}", e))),
			}
		} else {
			Ok(Some(Value::Null))
		} // Mimic "cannot rename"
	}

	async fn provide_rename_edits(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		new_name:String,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<WorkspaceEditDto>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::Rename)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(position_dto),
				new_name,
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!("{}$provideRenameEdits", ProxyTarget::ExtHostLanguageFeatures.target_prefix());
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 10000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize WorkspaceEditDto: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for renameEdits: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_folding_ranges(
		&self,
		document_uri:Url,
		language_id:String,
		context_dto:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<FoldingRangeDto>>, CommonError> {
		provide_feature!(
			self,
			"provideFoldingRanges",
			document_uri,
			language_id,
			CommonProviderType::FoldingRange,
			|h, uri, _pos_ignored:Value, ctx:Value, token| vec![json!(h), uri, ctx, token], /* Folding takes
			                                                                                 * context, not position */
			Vec<FoldingRangeDto>,
			5000
		)
	}

	async fn provide_selection_ranges(
		&self,
		document_uri:Url,
		language_id:String,
		positions_dto:Vec<PositionDto>,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<SelectionRangeDto>>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::SelectionRange)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(positions_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!(
				"{}$provideSelectionRanges",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 5000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<SelectionRangeDto>: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for selectionRanges: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_linked_editing_ranges(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<LinkedEditingRangesDto>, CommonError> {
		provide_feature!(
			self,
			"provideLinkedEditingRanges",
			document_uri,
			language_id,
			CommonProviderType::LinkedEditingRange,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			LinkedEditingRangesDto,
			5000
		)
	}

	async fn provide_document_semantic_tokens(
		&self,
		document_uri:Url,
		language_id:String,
		previous_result_id:Option<String>,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<SemanticTokensDto>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::SemanticTokens)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				previous_result_id,
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!(
				"{}$provideDocumentSemanticTokens",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 10000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize SemanticTokensDto: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for semantic tokens: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_document_semantic_tokens_edits(
		&self,
		document_uri:Url,
		language_id:String,
		previous_result_id:String,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::SemanticTokens)
			.await?; // Same provider type
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				previous_result_id,
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			// Cocoon's $provideDocumentSemanticTokens is expected to return
			// SemanticTokensEditsDto if previousResultId is provided and valid.
			let rpc_method = format!(
				"{}$provideDocumentSemanticTokens",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 10000)
				.await
			{
				Ok(v) if !v.is_null() => Ok(Some(v)), // Forward Value (SemanticTokensDto | SemanticTokensEditsDto)
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for semantic tokens edits: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_document_range_semantic_tokens(
		&self,
		document_uri:Url,
		language_id:String,
		range_dto:RangeDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<SemanticTokensDto>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::SemanticTokensRange)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(range_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!(
				"{}$provideDocumentRangeSemanticTokens",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 10000)
				.await
			{
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize SemanticTokensDto (range): {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for range semantic tokens: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn prepare_call_hierarchy(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		provide_feature!(
			self,
			"prepareCallHierarchy",
			document_uri,
			language_id,
			CommonProviderType::CallHierarchy,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			Vec<HierarchyItemDto>,
			10000
		)
	}

	async fn provide_call_hierarchy_incoming_calls(
		&self,
		_ignored_sidecar_id:String,
		item_dto:HierarchyItemDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<IncomingCallDto>>, CommonError> {
		let target_sidecar_id = self
			.get_sidecar_id_for_hierarchy_session(&item_dto._session_id, CommonProviderType::CallHierarchy)
			.await?;
		let rpc_params = json!([json!(item_dto), cancellation_token_id_val.unwrap_or(Value::Null)]);
		let rpc_method = format!(
			"{}$provideCallHierarchyIncomingCalls",
			ProxyTarget::ExtHostLanguageFeatures.target_prefix()
		);
		let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
		match ipc
			.send_request_to_sidecar(target_sidecar_id, rpc_method, rpc_params, 10000)
			.await
		{
			Ok(v) if !v.is_null() => {
				serde_json::from_value(v)
					.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<IncomingCallDto>: {}", e)))
					.map(Some)
			},
			Ok(_) => Ok(None),
			Err(e) => Err(CommonError::IpcError(format!("RPC for incoming calls: {}", e))),
		}
	}

	async fn provide_call_hierarchy_outgoing_calls(
		&self,
		_ignored_sidecar_id:String,
		item_dto:HierarchyItemDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<OutgoingCallDto>>, CommonError> {
		let target_sidecar_id = self
			.get_sidecar_id_for_hierarchy_session(&item_dto._session_id, CommonProviderType::CallHierarchy)
			.await?;
		let rpc_params = json!([json!(item_dto), cancellation_token_id_val.unwrap_or(Value::Null)]);
		let rpc_method = format!(
			"{}$provideCallHierarchyOutgoingCalls",
			ProxyTarget::ExtHostLanguageFeatures.target_prefix()
		);
		let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
		match ipc
			.send_request_to_sidecar(target_sidecar_id, rpc_method, rpc_params, 10000)
			.await
		{
			Ok(v) if !v.is_null() => {
				serde_json::from_value(v)
					.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<OutgoingCallDto>: {}", e)))
					.map(Some)
			},
			Ok(_) => Ok(None),
			Err(e) => Err(CommonError::IpcError(format!("RPC for outgoing calls: {}", e))),
		}
	}

	async fn prepare_type_hierarchy(
		&self,
		document_uri:Url,
		language_id:String,
		position_dto:PositionDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		provide_feature!(
			self,
			"prepareTypeHierarchy",
			document_uri,
			language_id,
			CommonProviderType::TypeHierarchy,
			|h, uri, pos, _ctx:Value, token| vec![json!(h), uri, json!(pos), token],
			Vec<HierarchyItemDto>,
			10000
		)
	}

	async fn provide_type_hierarchy_supertypes(
		&self,
		_ignored_sidecar_id:String,
		item_dto:HierarchyItemDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		let target_sidecar_id = self
			.get_sidecar_id_for_hierarchy_session(&item_dto._session_id, CommonProviderType::TypeHierarchy)
			.await?;
		let rpc_params = json!([json!(item_dto), cancellation_token_id_val.unwrap_or(Value::Null)]);
		let rpc_method = format!(
			"{}$provideTypeHierarchySupertypes",
			ProxyTarget::ExtHostLanguageFeatures.target_prefix()
		);
		let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
		match ipc
			.send_request_to_sidecar(target_sidecar_id, rpc_method, rpc_params, 10000)
			.await
		{
			Ok(v) if !v.is_null() => {
				serde_json::from_value(v)
					.map_err(|e| {
						CommonError::IpcError(format!("Deserialize Vec<HierarchyItemDto> (supertypes): {}", e))
					})
					.map(Some)
			},
			Ok(_) => Ok(None),
			Err(e) => Err(CommonError::IpcError(format!("RPC for supertypes: {}", e))),
		}
	}

	async fn provide_type_hierarchy_subtypes(
		&self,
		_ignored_sidecar_id:String,
		item_dto:HierarchyItemDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		let target_sidecar_id = self
			.get_sidecar_id_for_hierarchy_session(&item_dto._session_id, CommonProviderType::TypeHierarchy)
			.await?;
		let rpc_params = json!([json!(item_dto), cancellation_token_id_val.unwrap_or(Value::Null)]);
		let rpc_method = format!(
			"{}$provideTypeHierarchySubtypes",
			ProxyTarget::ExtHostLanguageFeatures.target_prefix()
		);
		let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
		match ipc
			.send_request_to_sidecar(target_sidecar_id, rpc_method, rpc_params, 10000)
			.await
		{
			Ok(v) if !v.is_null() => {
				serde_json::from_value(v)
					.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<HierarchyItemDto> (subtypes): {}", e)))
					.map(Some)
			},
			Ok(_) => Ok(None),
			Err(e) => Err(CommonError::IpcError(format!("RPC for subtypes: {}", e))),
		}
	}

	async fn provide_inlay_hints(
		&self,
		document_uri:Url,
		language_id:String,
		range_dto:RangeDto,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Vec<InlayHintDto>>, CommonError> {
		let providers = self
			.get_providers_for_document_internal(&document_uri, &language_id, CommonProviderType::InlayHints)
			.await?;
		if let Some(p_reg) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_reg.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(range_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);
			let rpc_method = format!("{}$provideInlayHints", ProxyTarget::ExtHostLanguageFeatures.target_prefix());
			let ipc:Arc<dyn IpcProvider + Send + Sync> = self.require();
			match ipc
				.send_request_to_sidecar(p_reg.sidecar_id.clone(), rpc_method, rpc_params, 7000)
				.await
			{
				Ok(v) if !v.is_null() => {
					// Cocoon sends IInlayHintsDto { hints: IInlayHintDto[], cacheId?: number }
					// We need Vec<InlayHintDto>. If `v` is the IInlayHintsDto.
					let list_val = v.get("hints").cloned().unwrap_or_else(|| Value::Array(vec![]));
					serde_json::from_value(list_val)
						.map_err(|e| CommonError::IpcError(format!("Deserialize Vec<InlayHintDto>: {}", e)))
						.map(Some)
				},
				Ok(_) => Ok(None),
				Err(e) => Err(CommonError::IpcError(format!("RPC for inlay hints: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn resolve_inlay_hint(
		&self,
		provider_handle:u32,
		_ignored_sidecar_id:String,
		hint_dto_val:Value,
		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<InlayHintDto>, CommonError> {
		resolve_feature_item!(
			self,
			"resolveInlayHint",
			provider_handle,
			hint_dto_val,
			cancellation_token_id_val,
			CommonProviderType::InlayHints,
			InlayHintDto,
			5000
		)
	}
}

impl MountainEnvironment {
	// Helper specific to hierarchy
	async fn get_sidecar_id_for_hierarchy_session(
		&self,
		session_id:&str,
		expected_type:CommonProviderType,
	) -> Result<String, CommonError> {
		let app_state = self.get_app_state();
		let sessions_guard = app_state
			.active_hierarchy_sessions
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;
		if let Some(session_ctx) = sessions_guard.get(session_id) {
			if session_ctx.provider_type == expected_type {
				Ok(session_ctx.original_sidecar_id.clone())
			} else {
				Err(CommonError::InvalidArg(
					"session_id".to_string(),
					format!("Session {} is not for {:?} hierarchy", session_id, expected_type),
				))
			}
		} else {
			// Fallback or error if session not found
			warn!(
				"[Env Hierarchy] Session ID '{}' not found in active sessions. Defaulting to 'cocoon-main'. This may \
				 be incorrect.",
				session_id
			);
			Ok("cocoon-main".to_string()) // This is a guess/fallback
			// Err(CommonError::InvalidArg("session_id".to_string(),
			// format!("Hierarchy session {} not found", session_id)))
		}
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}
