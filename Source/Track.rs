// ---------------------------------------------------------------------------------------------
// Mountain Track - Command and Request Dispatcher 
// --------------------------------------------------------------------------------------------
// Acts as the central routing hub for actions within Mountain, originating from
// both the Sky frontend (via Tauri `invoke`) and sidecar processes like Cocoon
// (via the Vine IPC layer). Its primary role is to translate these incoming
// commands and requests into abstract `ActionEffect`s (defined in
// `Land_Common`) or to route them to direct handler functions or RPC struct
// methods. `ActionEffect`s are then dispatched to the `AppRuntime` for
// execution.
//
// Responsibilities:
// - `dispatch_command`: Entry point for Sky frontend commands.
// - `dispatch_sidecar_request`: Entry point for sidecar IPC messages.
// - Parsing command/method names and arguments.
// - Prioritizing direct handling for specific notifications.
// - Mapping incoming commands/RPC methods to `ActionEffect`s (preferred).
// - Falling back to RPC handler structs (`rpc.rs`) or direct `handlers::*`
//   functions.
// - Invoking `AppRuntime::run(effect)` for `ActionEffect`s.
// - Formatting responses and errors for Sky or Vine.
// - Providing specific Tauri commands for fine-grained frontend interactions
//   (e.g., language features).
// --------------------------------------------------------------------------------------------
use std::{collections::HashMap, path::PathBuf, sync::Arc};

// Import effect constructors and DTOs from Land_Common
use Land_Common::{
	command_effects,

	config_effects::{self, ConfigurationTarget, IConfigurationOverrides},

	diagnostics_effects,

	documents_effects,

	effect::ActionEffect,

	errors::CommonError,

	fs_effects::{self, FsReader},     // FsReader for generic effect wrapper type
	ipc_effects::{self, ProxyTarget}, // Added ProxyTarget for dispatch_sidecar_request
	language_feature_effects::{
		self,

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

		LinkDto,

		LinkedEditingRangesDto,

		LinksListDto,

		LocationLinkDto,

		OutgoingCallDto,

		PositionDto,

		ProviderOptionsDto, // For ProviderRegistration
		ProviderType as CommonLangProviderType,

		RangeDto,

		SelectionRangeDto,

		SemanticTokensDto,

		SignatureHelpContextDto,

		SignatureHelpResultDto,

		SuggestResultDto,

		TextEditDto,

		WorkspaceEditDto,

		WorkspaceSymbolDto,

		prepare_call_hierarchy_effect,

		prepare_rename_effect,

		prepare_type_hierarchy_effect,

		provide_call_hierarchy_incoming_calls_effect,

		provide_call_hierarchy_outgoing_calls_effect,

		// Effect constructors for specific language features:
		provide_code_actions_effect,

		provide_code_lenses_effect,

		provide_completions_effect,

		provide_document_formatting_edits_effect,

		provide_document_highlights_effect,

		provide_document_links_effect,

		provide_document_semantic_tokens_edits_effect,

		provide_document_semantic_tokens_effect,

		provide_document_symbols_effect,

		provide_folding_ranges_effect,

		provide_hover_effect,

		provide_inlay_hints_effect,

		provide_linked_editing_ranges_effect,

		provide_references_effect,

		provide_rename_edits_effect,

		provide_selection_ranges_effect,

		provide_signature_help_effect,

		provide_type_hierarchy_subtypes_effect,

		provide_type_hierarchy_supertypes_effect,

		provide_workspace_symbols_effect,

		resolve_code_action_effect,

		resolve_code_lens_effect,

		resolve_completion_item_for_list_effect, // Used list_cache_id version
		resolve_document_link_effect,

		resolve_inlay_hint_effect,
	},

	output_effects,

	secrets_effects,

	storage_effects,

	ui_effects,

	workspace_effects::{self, apply_workspace_edit_effect},
};
// Constants for frontend command names
use Land_Echo;
// Logging
use log::{debug, error, info, trace, warn};
use serde::Deserialize; // For deserializing Tauri command args
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime as TauriRuntime, State, Window, command};
// For handling URIs in effect parameters
use url::Url;

use crate::{
	app_state::AppState, // For getting language_id
	handlers::{self, error_utils, language_features::MainThreadLanguageFeaturesHandler, sky_configuration},

	rpc,

	runtime::AppRuntime,
};

// --- Error Handling Abstraction ---
fn create_parameter_parse_error_string(
	method_name:&str,

	param_name:&str,

	expected_type:&str,

	index:Option<usize>,
) -> String {
	error_utils::rpc_param_error_string(method_name, param_name, expected_type, index)
}

fn map_common_error_to_rpc_error_string(e:CommonError, operation_context:&str) -> String {
	error_utils::map_common_error_to_rpc_string(e, operation_context)
}

// --- Helper to get language_id for a URI ---
fn get_language_id_for_uri(
	app_handle:&AppHandle<impl TauriRuntime>,

	uri:&Url,

	command_context:&str,
) -> Result<String, String> {
	let app_state = app_handle.state::<AppState>();

	let open_docs_guard = app_state.open_documents.lock().map_err(|e| {
		error_utils::format_app_state_lock_error(&format!("open_documents for {} langId", command_context), e)
	})?;

	open_docs_guard
		.get(uri.as_str())
		.map(|ds| ds.language_id.clone())
		.ok_or_else(|| {
			error_utils::rpc_error_string(
				format!("Document not found for {}: {}", command_context, uri),
				Some(&format!("ENODOC_{}", command_context.to_uppercase())),
			)
		})
}

// --- Frontend Command Dispatcher (`#[tauri::command]`) ---
#[command]
pub async fn dispatch_command<R:TauriRuntime>(
	app_handle:AppHandle<R>,

	window:Window<R>,

	runtime:State<'_, Arc<AppRuntime>>,

	command:String,

	args:Value,
) -> Result<Value, String> {
	info!("[Track FrontendCmd Dispatch] Command: '{}'", command);

	trace!("[Track FrontendCmd Dispatch] Argument: {:?}", args);

	match create_effect_for_frontend_command(&app_handle, &window, &command, args) {
		Ok(effect_to_run) => {
			runtime.run(effect_to_run).await.map_err(|common_err| {
				error!(
					"[Track FrontendCmd Dispatch] Error running effect for '{}': {}",
					command, common_err
				);

				map_common_error_to_rpc_error_string(common_err, &format!("frontend_cmd_exec_{}", command))
			})
		},

		Err(effect_creation_err_str) => {
			error!(
				"[Track FrontendCmd Dispatch] Error creating effect for '{}': {}",
				command, effect_creation_err_str
			);

			Err(effect_creation_err_str)
		},
	}
}

// --- Sidecar Request/Notification Dispatcher (Called by Vine) ---
pub async fn dispatch_sidecar_request<R:TauriRuntime>(
	app_handle:AppHandle<R>,

	_window:Window<R>, // Often unused by backend handlers
	runtime:State<'_, Arc<AppRuntime>>,

	sidecar_id:String,

	request_message_val:Value,
) -> Result<Value, String> {
	let rpc_method_name = request_message_val.get("method").and_then(Value::as_str).unwrap_or("");

	let rpc_params_val = request_message_val.get("params").cloned().unwrap_or(Value::Null);

	info!(
		"[Track SidecarReq Dispatch] From '{}': Method='{}'",
		sidecar_id, rpc_method_name
	);

	trace!(
		"[Track SidecarReq Dispatch] Params (type='{:?}'): {}...",
		rpc_params_val.kind(),
		rpc_params_val.to_string().chars().take(100).collect::<String>()
	);

	// --- 1. Prioritize Direct Handling for Specific Notifications ---
	if rpc_method_name.starts_with("terminal_") && rpc_method_name != "$createTerminal" {
		debug!(
			"[Track SidecarReq Dispatch] Routing terminal env notification '{}' directly.",
			rpc_method_name
		);

		return match rpc_method_name {
			"terminal_setEnvironmentVariable" => {
				handlers::terminal::handle_set_environment_variable_contribution(app_handle, rpc_params_val).await
			},

			"terminal_deleteEnvironmentVariable" => {
				handlers::terminal::handle_delete_environment_variable_contribution(app_handle, rpc_params_val).await
			},

			"terminal_clearEnvironmentVariableCollection" => {
				handlers::terminal::handle_clear_environment_variable_collection_contributions(
					app_handle,
					rpc_params_val,
				)
				.await
			},

			_ => {
				warn!(
					"[Track SidecarReq Dispatch] Unknown direct terminal notification: {}",
					rpc_method_name
				);

				Err(error_utils::rpc_error_string(
					format!("Unknown direct terminal notification: {}", rpc_method_name),
					Some("ENOSYS_TERM_NOTIF_UNKNOWN"),
				))
			},
		};
	}

	match rpc_method_name {
		"$log" | "$logExtensionHostActivation" | "$logExtensionHostRequest" => {
			let rpc_log_handler = rpc::MainThreadLogHandler { app_handle, runtime:runtime.inner().clone() };

			return rpc_log_handler.log(rpc_params_val).await;
		},

		"$onWillActivateExtension"
		| "$onDidActivateExtension"
		| "$onExtensionActivationError"
		| "$onExtensionRuntimeError" => {
			let params_as_array = rpc_params_val.as_array().cloned().unwrap_or_default();

			return handlers::extension_status::handle_extension_host_status_notification(
				app_handle,
				rpc_method_name,
				Value::Array(params_as_array),
			)
			.await;
		},

		_ => { /* Continue */ },
	}

	// --- 2. Attempt Effect Creation ---
	let params_array_for_effects = rpc_params_val
		.as_array()
		.cloned()
		.unwrap_or_else(|| vec![rpc_params_val.clone()]);

	match create_effect_for_sidecar_request(&sidecar_id, rpc_method_name, params_array_for_effects.clone()) {
		Ok(effect_to_run) => {
			debug!(
				"[Track SidecarReq Dispatch] Mapped RPC method '{}' to ActionEffect. Running...",
				rpc_method_name
			);

			return runtime.run(effect_to_run).await.map_err(|common_err| {
				error!(
					"[Track SidecarReq Dispatch] Error running effect for '{}': {}",
					rpc_method_name, common_err
				);

				map_common_error_to_rpc_error_string(common_err, &format!("sidecar_effect_exec_{}", rpc_method_name))
			});
		},

		Err(EffectCreationError::NoEffectMapping) => {
			debug!(
				"[Track SidecarReq Dispatch] No direct ActionEffect for '{}'. Attempting RPC fallback.",
				rpc_method_name
			);
		},

		Err(EffectCreationError::ParamParseError(err_str)) => {
			error!(
				"[Track SidecarReq Dispatch] Param parsing error for '{}': {}",
				rpc_method_name, err_str
			);

			return Err(err_str);
		},
	}

	// --- 3. Fallback to Direct RPC Handler Methods ---
	let rpc_handler_runtime_clone = runtime.inner().clone();

	if rpc_method_name.starts_with(&format!("{}$", ProxyTarget::MainThreadLanguageFeatures.target_prefix())) {
		let handler = MainThreadLanguageFeaturesHandler::new(app_handle.clone());

		let method_on_handler = rpc_method_name
			.trim_start_matches(&format!("{}$", ProxyTarget::MainThreadLanguageFeatures.target_prefix()));

		// Note: rpc_params_val IS the array [handle, selectorDto, optionsDto?,

		// extensionIdDto?]
		return match method_on_handler {
			"registerHoverProvider" => handler.registerHoverProvider(&sidecar_id, rpc_params_val).await,

			"registerCompletionProvider" | "registerCompletionsProvider" => {
				handler.registerCompletionItemProvider(&sidecar_id, rpc_params_val).await
			},

			"registerDefinitionSupport" => handler.registerDefinitionProvider(&sidecar_id, rpc_params_val).await,

			"registerDeclarationSupport" => handler.registerDeclarationProvider(&sidecar_id, rpc_params_val).await,

			"registerImplementationSupport" => {
				handler.registerImplementationProvider(&sidecar_id, rpc_params_val).await
			},

			"registerTypeDefinitionSupport" => {
				handler.registerTypeDefinitionProvider(&sidecar_id, rpc_params_val).await
			},

			"registerCodeLensSupport" => handler.registerCodeLensProvider(&sidecar_id, rpc_params_val).await,

			"registerCodeActionSupport" => handler.registerCodeActionProvider(&sidecar_id, rpc_params_val).await,

			"registerDocumentFormattingSupport" => {
				handler
					.registerDocumentFormattingEditProvider(&sidecar_id, rpc_params_val)
					.await
			},

			"registerRangeFormattingSupport" => {
				handler
					.registerDocumentRangeFormattingEditProvider(&sidecar_id, rpc_params_val)
					.await
			},

			"registerOnTypeFormattingSupport" => {
				handler.registerOnTypeFormattingEditProvider(&sidecar_id, rpc_params_val).await
			},

			"registerDocumentHighlightProvider" => {
				handler.registerDocumentHighlightProvider(&sidecar_id, rpc_params_val).await
			},

			"registerDocumentLinkProvider" => handler.registerDocumentLinkProvider(&sidecar_id, rpc_params_val).await,

			"registerDocumentColorProvider" => handler.registerDocumentColorProvider(&sidecar_id, rpc_params_val).await,

			"registerFoldingRangeProvider" | "registerFoldingRangeSupport" => {
				handler.registerFoldingRangeProvider(&sidecar_id, rpc_params_val).await
			},

			"registerReferenceSupport" => handler.registerReferenceProvider(&sidecar_id, rpc_params_val).await,

			"registerRenameSupport" => handler.registerRenameProvider(&sidecar_id, rpc_params_val).await,

			"registerSignatureHelpProvider" => handler.registerSignatureHelpProvider(&sidecar_id, rpc_params_val).await,

			"registerNavigateTypeSupport" => handler.registerWorkspaceSymbolProvider(&sidecar_id, rpc_params_val).await,

			"registerDocumentSymbolProvider" => {
				handler.registerDocumentSymbolProvider(&sidecar_id, rpc_params_val).await
			},

			"registerSelectionRangeProvider" | "registerSelectionRangeSupport" => {
				handler.registerSelectionRangeProvider(&sidecar_id, rpc_params_val).await
			},

			"registerCallHierarchyProvider" | "registerCallHierarchySupport" => {
				handler.registerCallHierarchyProvider(&sidecar_id, rpc_params_val).await
			},

			"registerTypeHierarchyProvider" | "registerTypeHierarchySupport" => {
				handler.registerTypeHierarchyProvider(&sidecar_id, rpc_params_val).await
			},

			"registerLinkedEditingRangeProvider" => {
				handler.registerLinkedEditingRangeProvider(&sidecar_id, rpc_params_val).await
			},

			"registerInlayHintsProvider" => handler.registerInlayHintsProvider(&sidecar_id, rpc_params_val).await,

			"unregister" | "unregisterProvider" => handler.unregisterProvider(&sidecar_id, rpc_params_val).await, /* This is handled by effect now primarily */
			"emitCodeLensEvent" => handler.emitCodeLensEvent(&sidecar_id, rpc_params_val).await,

			_ => {
				error!(
					"[Track SidecarReq Dispatch] Unhandled MainThreadLanguageFeatures method: '{}'",
					rpc_method_name
				);

				Err(error_utils::rpc_error_string(
					format!("Unknown MainThreadLanguageFeatures method: {}", rpc_method_name),
					Some("ENOSYS_LANG_FEAT_METH_TRACK"),
				))
			},
		};
	}

	match rpc_method_name {
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" => {
			let handler = rpc::MainThreadCommandsHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$executeCommand" => handler.executeCommand(rpc_params_val).await,

				"$getCommands" => handler.getCommands(rpc_params_val).await,

				"$registerCommand" => handler.registerCommand(rpc_params_val).await,

				"$unregisterCommand" => handler.unregisterCommand(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		"$resolveWorkspaceFolder" => {
			let handler = rpc::MainThreadWorkspaceHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.resolveWorkspaceFolder(rpc_params_val).await
		},

		"$findFiles" => handlers::workspace::handle_find_files(app_handle, rpc_params_val).await,

		"$showMessage" => {
			let handler = rpc::MainThreadMessageHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.showMessage(rpc_params_val).await
		},

		"$showOpenDialog" | "$showSaveDialog" => {
			let handler = rpc::MainThreadDialogsHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$showOpenDialog" => handler.showOpenDialog(rpc_params_val).await,

				"$showSaveDialog" => handler.showSaveDialog(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		"$focusWindow" => {
			let handler = rpc::MainThreadWindowHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.focusWindow(rpc_params_val).await
		},

		"$setEntry" | "$disposeEntry" if rpc_method_name == "$setEntry" || rpc_method_name == "$disposeEntry" => {
			let handler = rpc::MainThreadStatusBarHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$setEntry" => handler.setEntry(rpc_params_val).await,

				"$disposeEntry" => handler.disposeEntry(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename"
		| "$copy" => {
			let fs_api_handler = rpc::MainThreadFileSystemApiHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$stat" => fs_api_handler.stat(rpc_params_val).await,

				"$readDirectory" => fs_api_handler.read_directory(rpc_params_val).await,

				"$readFile" => fs_api_handler.read_file(rpc_params_val).await,

				"$writeFile" => fs_api_handler.write_file(rpc_params_val).await,

				"$createDirectory" => fs_api_handler.create_directory(rpc_params_val).await,

				"$delete" => fs_api_handler.delete(rpc_params_val).await,

				"$rename" => fs_api_handler.rename(rpc_params_val).await,

				"$copy" => fs_api_handler.copy(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		"$tryOpenDocument" => handlers::documents::handle_try_open_document(app_handle, rpc_params_val).await,

		"$tryCreateDocument" => handlers::documents::handle_try_create_document(app_handle, rpc_params_val).await,

		"$trySaveDocument" => {
			let uri_dto_val = rpc_params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
				create_parameter_parse_error_string(
					rpc_method_name,
					"uriComponents (args[0])",
					"Value::Object",
					Some(0),
				)
			})?;

			handlers::documents::handle_try_save_document(app_handle, uri_dto_val).await
		},

		"$trySaveDocumentAs" => {
			let original_uri_dto_val = rpc_params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
				create_parameter_parse_error_string(
					rpc_method_name,
					"originalUriComponents (args[0])",
					"Value::Object",
					Some(0),
				)
			})?;

			handlers::documents::handle_try_save_document_as(app_handle, original_uri_dto_val).await
		},

		"$saveAll" => {
			let include_untitled_bool = rpc_params_val
				.as_array()
				.and_then(|a| a.get(0))
				.and_then(Value::as_bool)
				.unwrap_or(true);

			handlers::documents::handle_save_all(app_handle, include_untitled_bool).await
		},

		_ if is_output_method_fallback_candidate(rpc_method_name) => {
			match rpc_method_name {
				"$register" => handlers::output::handle_register_output_channel(app_handle, rpc_params_val).await,

				"$append" => handlers::output::handle_append_to_output_channel(app_handle, rpc_params_val).await,

				"$replace" => handlers::output::handle_replace_output_channel_content(app_handle, rpc_params_val).await,

				"$reveal" => handlers::output::handle_reveal_output_channel(app_handle, rpc_params_val).await,

				"$close" => handlers::output::handle_close_output_channel_view(app_handle, rpc_params_val).await,

				_ => {
					error!(
						"[Track SidecarReq Dispatch] Unhandled output method in fallback: '{}'",
						rpc_method_name
					);

					Err(error_utils::rpc_error_string(
						format!("Output method '{}' not routed in fallback.", rpc_method_name),
						Some("ENOSYS_OUT_FALLBACK_ROUTE"),
					))
				},
			}
		},

		"$changeMany" => handlers::diagnostics::handle_change_many(app_handle, rpc_params_val).await,

		"$getDiagnostics" => handlers::diagnostics::handle_get_diagnostics(app_handle, rpc_params_val).await,

		"$createTerminal" | "$show" | "$hide" | "$sendText" => {
			let terminal_rpc_handler =
				rpc::MainThreadTerminalServiceHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$createTerminal" => terminal_rpc_handler.createTerminal(rpc_params_val).await,

				"$show" => terminal_rpc_handler.show(rpc_params_val).await,

				"$hide" => terminal_rpc_handler.hide(rpc_params_val).await,

				"$sendText" => terminal_rpc_handler.sendText(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		"$dispose" if rpc_params_val.as_array().and_then(|a| a.get(0)?.as_u64()).is_some() => {
			info!("[Track SidecarReq Dispatch] Assuming '$dispose' with u64 param is for Terminal (fallback).");

			let terminal_rpc_handler =
				rpc::MainThreadTerminalServiceHandler { app_handle, runtime:rpc_handler_runtime_clone };

			terminal_rpc_handler.dispose(rpc_params_val).await
		},

		"$dispose" if rpc_params_val.as_array().and_then(|a| a.get(0)?.as_str()).is_some() => {
			info!(
				"[Track SidecarReq Dispatch] Assuming '$dispose' with string param is for Output Channel (fallback)."
			);

			handlers::output::handle_dispose_output_channel(app_handle, rpc_params_val).await
		},

		"$clear" if rpc_params_val.as_array().and_then(|a| a.get(0)?.as_str()).is_some() => {
			info!("[Track SidecarReq Dispatch] Assuming '$clear' with string param is for Output Channel (fallback).");

			handlers::output::handle_clear_output_channel(app_handle, rpc_params_val).await
		},

		_ => {
			error!(
				"[Track SidecarReq Dispatch] Unhandled RPC method '{}' from sidecar '{}'. No effect or RPC handler.",
				rpc_method_name, sidecar_id
			);

			Err(error_utils::rpc_error_string(
				format!("RPC method '{}' not implemented.", rpc_method_name),
				Some("ENOSYS_TRACK_UNHANDLED_FINAL"),
			))
		},
	}
}

/// Helper to check if a method name is a candidate for output channel fallback
/// logic.
fn is_output_method_fallback_candidate(method_name:&str) -> bool {
	matches!(method_name, "$register" | "$append" | "$replace" | "$reveal" | "$close")
}

/// Represents errors that can occur during the creation of an `ActionEffect`.
enum EffectCreationError {
	NoEffectMapping,

	ParamParseError(String), // String is already a JSON-RPC error string
}

// --- Effect Creation Logic ---
fn create_effect_for_frontend_command<R:TauriRuntime>(
	_app_handle:&AppHandle<R>,

	_window:&Window<R>,

	command_id_str:&str,

	args_val:Value,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> {
	let frontend_param_err_fn = |param_name:&str, expected_type:&str| -> String {
		create_parameter_parse_error_string(command_id_str, param_name, expected_type, None)
	};

	let get_string_arg_from_obj = |key:&str| {
		args_val
			.get(key)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| frontend_param_err_fn(key, "string"))
	};

	let get_path_buf_arg_from_obj = |key:&str| get_string_arg_from_obj(key).map(PathBuf::from);

	let get_i64_arg_from_obj = |key:&str| {
		args_val
			.get(key)
			.and_then(Value::as_i64)
			.ok_or_else(|| frontend_param_err_fn(key, "i64 number"))
	};

	let get_bool_arg_from_obj =
		|key:&str, default_val:bool| args_val.get(key).and_then(Value::as_bool).unwrap_or(default_val);

	let get_optional_value_arg_from_obj = |key:&str| args_val.get(key).cloned();

	let get_required_value_arg_from_obj = |key:&str| {
		args_val
			.get(key)
			.cloned()
			.ok_or_else(|| frontend_param_err_fn(key, "JSON value"))
	};

	trace!(
		"[Track CreateEffect Frontend] Command='{}', Argument='{:?}'",
		command_id_str, args_val
	);

	match command_id_str {
		Land_Echo::REQUEST_READ_FILE => {
			let file_path = get_path_buf_arg_from_obj("path")?;

			let read_file_effect = fs_effects::read_file(file_path);

			Ok(ActionEffect::new(Arc::new(move |runtime_accessor| {
				let effect_clone = read_file_effect.clone();

				Box::pin(async move {
					runtime_accessor
						.run(effect_clone)
						.await
						.map(|bytes_vec| json!(base64::encode(bytes_vec)))
				})
			})))
		},

		Land_Echo::REQUEST_WRITE_FILE => {
			Ok(fs_effects::write_file_string(
				get_path_buf_arg_from_obj("path")?,
				get_string_arg_from_obj("content")?,
				get_bool_arg_from_obj("create", true),
				get_bool_arg_from_obj("overwrite", true),
			))
		},

		Land_Echo::REQUEST_NEW_FILE => {
			Ok(fs_effects::create_file(
				get_path_buf_arg_from_obj("parentDir")?.join(get_string_arg_from_obj("name")?),
			))
		},

		Land_Echo::REQUEST_NEW_FOLDER => {
			Ok(fs_effects::create_directory(
				get_path_buf_arg_from_obj("parentDir")?.join(get_string_arg_from_obj("name")?),
				true,
			))
		},

		Land_Echo::REQUEST_DELETE_PATH => {
			Ok(fs_effects::delete(
				get_path_buf_arg_from_obj("path")?,
				get_bool_arg_from_obj("recursive", true),
				get_bool_arg_from_obj("useTrash", false),
			))
		},

		Land_Echo::REQUEST_RENAME_PATH => {
			let old_path_buf = get_path_buf_arg_from_obj("oldPath")?;

			let new_name_str = get_string_arg_from_obj("newName")?;

			let parent_dir_path = old_path_buf.parent().ok_or_else(|| {
				frontend_param_err_fn(
					"parent of oldPath",
					&format!("valid parent directory for '{}'", old_path_buf.display()),
				)
			})?;

			Ok(fs_effects::rename(
				old_path_buf,
				parent_dir_path.join(new_name_str),
				get_bool_arg_from_obj("overwrite", false),
			))
		},

		Land_Echo::REQUEST_COPY_PATH => {
			Ok(fs_effects::copy(
				get_path_buf_arg_from_obj("sourcePath")?,
				get_path_buf_arg_from_obj("targetParentDir")?.join(get_string_arg_from_obj("newName")?),
				get_bool_arg_from_obj("overwrite", false),
			))
		},

		Land_Echo::REQUEST_SAVE_FILE => {
			Ok(documents_effects::try_save_document(
				Url::parse(&get_string_arg_from_obj("uri")?)
					.map_err(|e_url| frontend_param_err_fn("uri (parse error)", &e_url.to_string()))?,
			))
		},

		Land_Echo::REQUEST_SAVE_FILE_AS => {
			Ok(documents_effects::try_save_document_as(
				Url::parse(&get_string_arg_from_obj("originalUri")?)
					.map_err(|e_url| frontend_param_err_fn("originalUri (parse error)", &e_url.to_string()))?,
				get_optional_value_arg_from_obj("newTargetUri")
					.and_then(|val| val.as_str().map(|s| Url::parse(s)))
					.transpose()
					.map_err(|e_url| frontend_param_err_fn("newTargetUri (parse error)", &e_url.to_string()))?,
			))
		},

		Land_Echo::REQUEST_APPLY_EDITOR_CHANGES => {
			Ok(documents_effects::apply_document_changes(
				Url::parse(&get_string_arg_from_obj("uri")?)
					.map_err(|e_url| frontend_param_err_fn("uri (parse error)", &e_url.to_string()))?,
				get_i64_arg_from_obj("versionId")?,
				get_required_value_arg_from_obj("changes")?,
				get_bool_arg_from_obj("isDirty", true),
				get_bool_arg_from_obj("isUndoing", false),
				get_bool_arg_from_obj("isRedoing", false),
			))
		},

		Land_Echo::REQUEST_OPEN_FILE => {
			Ok(documents_effects::try_open_document(
				get_required_value_arg_from_obj("uriComponents")?,
				get_optional_value_arg_from_obj("languageId").and_then(|v| v.as_str().map(String::from)),
				get_optional_value_arg_from_obj("content").and_then(|v| v.as_str().map(String::from)),
			))
		},

		Land_Echo::REQUEST_PROXY_EXT_HOST_CALL => {
			Ok(ipc_effects::proxy_call_to_sidecar(
				"cocoon-main".to_string(),
				get_required_value_arg_from_obj("callData")?,
			))
		},

		Land_Echo::REQUEST_ESTABLISH_HOST_CONNECTION => {
			Ok(ipc_effects::establish_host_connection("cocoon-main".to_string()))
		},

		Land_Echo::REQUEST_WS_SEND | Land_Echo::REQUEST_WS_CONNECT => {
			Err(error_utils::rpc_error_string(
				format!("WebSocket command '{}' not implemented via effect system.", command_id_str),
				Some("ENOSYS_WS_EFFECT"),
			))
		},

		_ => {
			Err(error_utils::rpc_error_string(
				format!("Unknown frontend command ID '{}'", command_id_str),
				Some("ENOSYS_CMD_UNKNOWN"),
			))
		},
	}
}

fn create_effect_for_sidecar_request(
	sidecar_id_str:&str,

	rpc_method_name:&str,

	params_vec:Vec<Value>,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, EffectCreationError> {
	let sidecar_param_err_fn = |param_name:&str, expected_type:&str, idx:usize| {
		EffectCreationError::ParamParseError(create_parameter_parse_error_string(
			rpc_method_name,
			param_name,
			expected_type,
			Some(idx),
		))
	};

	let get_string_param_at_idx = |idx:usize, name_for_err:&str| {
		params_vec
			.get(idx)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| sidecar_param_err_fn(name_for_err, "string", idx))
	};

	let get_u32_param_at_idx = |idx:usize, name_for_err:&str| {
		params_vec
			.get(idx)
			.and_then(Value::as_u64)
			.map(|v| v as u32)
			.ok_or_else(|| sidecar_param_err_fn(name_for_err, "u32 number", idx))
	};

	let get_optional_param_at_idx = |idx:usize| params_vec.get(idx).cloned();

	let get_required_param_at_idx = |idx:usize, name_for_err:&str| {
		params_vec
			.get(idx)
			.cloned()
			.ok_or_else(|| sidecar_param_err_fn(name_for_err, "JSON value", idx))
	};

	let lang_feat_reg_effect_adapter = |effect_u32:ActionEffect<Arc<AppRuntime>, CommonError, u32>| -> Result<
		ActionEffect<Arc<AppRuntime>, CommonError, Value>,
		EffectCreationError,
	> {
		Ok(ActionEffect::new(Arc::new(move |runtime_accessor| {
			let effect_clone = effect_u32.clone();

			Box::pin(async move { runtime_accessor.run(effect_clone).await.map(Value::from) })
		})))
	};

	let lang_feat_void_effect_adapter = |effect_void:ActionEffect<Arc<AppRuntime>, CommonError, ()>| -> Result<
		ActionEffect<Arc<AppRuntime>, CommonError, Value>,
		EffectCreationError,
	> {
		Ok(ActionEffect::new(Arc::new(move |runtime_accessor| {
			let effect_clone = effect_void.clone();

			Box::pin(async move { runtime_accessor.run(effect_clone).await.map(|_| Value::Null) })
		})))
	};

	trace!(
		"[Track CreateEffect Sidecar] Method='{}', NumParams={}, Sidecar='{}'",
		rpc_method_name,
		params_vec.len(),
		sidecar_id_str
	);

	// For $register...Provider methods, params are [cocoon_handle, selectorDto,

	// optionsDto?, extensionIdDto?] We need selectorDto (idx 1), optionsDto (idx
	// 2), and extensionIdDto (idx 3) for the generic register_provider effect.
	let selector_dto_for_reg = get_required_param_at_idx(1, "selector DTO");

	let options_dto_for_reg = get_optional_param_at_idx(2);

	let extension_id_dto_for_reg = get_required_param_at_idx(3, "extensionId DTO"); // Assuming always present for registrations

	match rpc_method_name {
		"$getConfiguration" => {
			Ok(config_effects::get_configuration(
				get_optional_param_at_idx(0).and_then(|v| v.as_str().map(String::from)),
				get_optional_param_at_idx(1).unwrap_or(Value::Null),
				get_optional_param_at_idx(2).and_then(Value::as_bool),
			))
		},

		"$updateConfigurationOption" => {
			Ok(config_effects::update_configuration(
				get_string_param_at_idx(1, "key")?,
				get_required_param_at_idx(2, "value")?,
				get_u32_param_at_idx(0, "target (ConfigurationTarget)")?,
				get_optional_param_at_idx(3).unwrap_or(Value::Null),
				get_optional_param_at_idx(4).and_then(Value::as_bool),
			))
		},

		"$removeConfigurationOption" => {
			Ok(config_effects::update_configuration(
				get_string_param_at_idx(1, "key")?,
				Value::Null,
				get_u32_param_at_idx(0, "target (ConfigurationTarget)")?,
				get_optional_param_at_idx(2).unwrap_or(Value::Null),
				get_optional_param_at_idx(3).and_then(Value::as_bool),
			))
		},

		"$inspect" => {
			Ok(config_effects::inspect_configuration_value(
				get_string_param_at_idx(0, "key")?,
				get_optional_param_at_idx(1).unwrap_or(Value::Null),
			))
		},

		"$getWorkspaceFolders" => Ok(workspace_effects::get_workspace_folders_info().map_value(|v_vec| json!(v_vec))),

		"$requestWorkspaceTrust" => Ok(workspace_effects::request_workspace_trust(get_optional_param_at_idx(0))),

		"$getValue" => {
			Ok(storage_effects::get_storage_item(get_required_param_at_idx(
				0,
				"storage target object {scope, key}",
			)?))
		},

		"$setValue" => {
			Ok(storage_effects::set_storage_item(
				get_required_param_at_idx(0, "storage target object {scope, key}")?,
				get_required_param_at_idx(1, "value to set")?,
			))
		},

		"$getPassword" => {
			Ok(secrets_effects::get_secret(
				get_string_param_at_idx(0, "extensionId")?,
				get_string_param_at_idx(1, "key")?,
			))
		},

		"$setPassword" => {
			Ok(secrets_effects::store_secret(
				get_string_param_at_idx(0, "extensionId")?,
				get_string_param_at_idx(1, "key")?,
				get_string_param_at_idx(2, "value")?,
			))
		},

		"$deletePassword" => {
			Ok(secrets_effects::delete_secret(
				get_string_param_at_idx(0, "extensionId")?,
				get_string_param_at_idx(1, "key")?,
			))
		},

		// Language Feature Provider Registrations (these are now mostly handled by the RPC handler fallback)
		// This section is reduced as MainThreadLanguageFeaturesHandler handles most $register calls.
		// Only $unregister might remain as a direct effect here if not in RPC handler.
		"$registerHoverProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Hover,
				selector_dto_for_reg?,
				sidecar_id_str.to_string(),
				extension_id_dto_for_reg?,
				options_dto_for_reg,
			))
		},

		"$registerCompletionItemProvider" | "$registerCompletionsProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Completion,
				selector_dto_for_reg?,
				sidecar_id_str.to_string(),
				extension_id_dto_for_reg?,
				options_dto_for_reg,
			))
		},

		// ... Add other $register... calls if they are to be effects and not RPC handlers
		// Ensure parameters match `language_feature_effects::register_provider`
		"$unregister" | "$unregisterProvider"
			if rpc_method_name != "$unregisterLogSink" && rpc_method_name != "$unregisterSerializer" =>
		{
			lang_feat_void_effect_adapter(language_feature_effects::unregister_provider(get_u32_param_at_idx(
				0,
				"provider_handle (Mountain-generated)",
			)?))
		},

		"$clear"
			if rpc_method_name == "$clear"
				&& params_vec.len() == 1
				&& params_vec.get(0).map_or(false, Value::is_string) =>
		{
			Ok(diagnostics_effects::clear_diagnostics(get_string_param_at_idx(0, "owner")?))
		},

		_ => Err(EffectCreationError::NoEffectMapping),
	}
}

// --- Tauri Commands for Specific Language Features (Sky -> Mountain) ---

#[command]
pub async fn mountain_get_workbench_configuration(
	app_handle:AppHandle<Wry>,
) -> Result<sky_dtos::SandboxConfigurationDto, String> {
	info!("[Track Command] mountain_get_workbench_configuration request.");

	Ok(sky_configuration::build_sandbox_configuration(&app_handle))
}

// Define sky_dtos if not already available (placeholder)
mod sky_dtos {

	use serde::Serialize;

	#[derive(Serialize, Debug, Default)]
	pub struct SandboxConfigurationDto {
		// Placeholder fields, actual fields would match VS Code's ISandboxConfiguration
		pub locale:Option<String>,

		pub user_data_path:Option<String>,

		pub machine_settings_path:Option<String>,
	}
}

#[derive(Deserialize, Debug)]
struct RequestHoverArgument {
	uri_string:String,

	line_number_0_based:u32,

	column_0_based:u32,
}

#[command]
pub async fn mountain_request_hover(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:RequestHoverArgument,
) -> Result<Option<HoverResultDto>, String> {
	info!("[Track Command] mountain_request_hover for URI: {}", args.uri_string);

	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_hover", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "hover")?;

	let position_dto = PositionDto { line_number:args.line_number_0_based, column:args.column_0_based };

	let effect = provide_hover_effect(target_uri, language_id, position_dto);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "hover_effect"))
}

#[derive(Deserialize, Debug)]
struct RequestCompletionsArgument {
	uri_string:String,

	line_number_0_based:u32,

	column_0_based:u32,

	context_dto:CompletionContextDto,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_completions(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:RequestCompletionsArgument,
) -> Result<Option<SuggestResultDto>, String> {
	info!("[Track Command] mountain_request_completions for URI: {}", args.uri_string);

	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_completions", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "completions")?;

	let position_dto = PositionDto { line_number:args.line_number_0_based, column:args.column_0_based };

	let effect = provide_completions_effect(
		target_uri,
		language_id,
		position_dto,
		args.context_dto,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "completions_effect"))
}

#[derive(Deserialize, Debug)]
struct ResolveCompletionItemArgumentSky {
	list_cache_id:u32,

	item_dto_to_resolve:Value, // ISuggestDataDto as Value
	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_resolve_completion_item(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ResolveCompletionItemArgumentSky,
) -> Result<Option<Value>, String> {
	info!(
		"[Track Command] mountain_resolve_completion_item for ListCacheID: {}",
		args.list_cache_id
	);

	trace!(
		"[Track Command] Item DTO to resolve for list {}: {:?}",
		args.list_cache_id, args.item_dto_to_resolve
	);

	let effect = resolve_completion_item_for_list_effect(
		args.list_cache_id,
		args.item_dto_to_resolve,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "resolve_completion_item_effect"))
}

#[derive(Deserialize, Debug)]
struct RequestCodeActionsArgument {
	uri_string:String,

	line_number_0_based_start:u32,

	column_0_based_start:u32,

	line_number_0_based_end:u32,

	column_0_based_end:u32,

	context_dto:CodeActionContextDto,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_code_actions(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:RequestCodeActionsArgument,
) -> Result<Option<CodeActionListDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_code_actions", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "code_actions")?;

	let range_or_selection_dto = json!({

		"startLineNumber": args.line_number_0_based_start,

		"startColumn": args.column_0_based_start,

		"endLineNumber": args.line_number_0_based_end,

		"endColumn": args.column_0_based_end,

	});

	let effect = provide_code_actions_effect(
		target_uri,
		language_id,
		range_or_selection_dto,
		args.context_dto,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "code_actions_effect"))
}

#[derive(Deserialize, Debug)]
struct ResolveCodeActionArgumentSky {
	list_cache_id:u32, // From item's cache_id[0]
	// Assuming MountainEnvironment or the effect can find the sidecar_id from the list_cache_id or item data
	action_dto_to_resolve:Value, // CodeActionDto as Value
	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_resolve_code_action(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ResolveCodeActionArgumentSky,
) -> Result<Option<CodeActionDto>, String> {
	// The effect now expects sidecar_id. For Sky-invoked resolve, sidecar_id might
	// need to be looked up by list_cache_id or item. This lookup logic would be in
	// MountainEnvironment or the effect itself if list_cache_id points to a
	// ProviderRegistration. For now, a placeholder sidecar_id is used as it was in
	// the snippet.
	let sidecar_id_for_resolve = "cocoon-main".to_string(); // Placeholder - this needs robust handling
	let effect = resolve_code_action_effect(
		args.list_cache_id,
		sidecar_id_for_resolve,
		args.action_dto_to_resolve,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "resolve_code_action_effect"))
}

#[derive(Deserialize, Debug)]
struct RequestCodeLensesArgument {
	uri_string:String,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_code_lenses(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:RequestCodeLensesArgument,
) -> Result<Option<CodeLensListDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_code_lenses", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "code_lenses")?;

	let effect = provide_code_lenses_effect(target_uri, language_id, args.cancellation_token_id_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "code_lenses_effect"))
}

#[derive(Deserialize, Debug)]
struct ResolveCodeLensArgumentSky {
	list_cache_id:u32,

	// Assuming sidecar_id can be derived
	lens_dto_to_resolve:Value, // CodeLensDto as Value
	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_resolve_code_lens(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ResolveCodeLensArgumentSky,
) -> Result<Option<CodeLensDto>, String> {
	let sidecar_id_for_resolve = "cocoon-main".to_string(); // Placeholder
	let effect = resolve_code_lens_effect(
		args.list_cache_id,
		sidecar_id_for_resolve,
		args.lens_dto_to_resolve,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "resolve_code_lens_effect"))
}

#[derive(Deserialize, Debug)]
struct DocumentSymbolsArgument {
	uri_string:String,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_document_symbols(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:DocumentSymbolsArgument,
) -> Result<Option<Vec<DocumentSymbolDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_document_symbols", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "doc_symbols")?;

	let effect = provide_document_symbols_effect(target_uri, language_id, args.cancellation_token_id_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "doc_symbols_effect"))
}

#[derive(Deserialize, Debug)]
struct WorkspaceSymbolsArgument {
	query:String,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_workspace_symbols(
	runtime:State<'_, Arc<AppRuntime>>,

	args:WorkspaceSymbolsArgument,
) -> Result<Option<Vec<WorkspaceSymbolDto>>, String> {
	let effect = provide_workspace_symbols_effect(args.query, args.cancellation_token_id_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "ws_symbols_effect"))
}

#[derive(Deserialize, Debug)]
struct SignatureHelpArgument {
	uri_string:String,

	line_number_0_based:u32,

	column_0_based:u32,

	context_dto:SignatureHelpContextDto,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_signature_help(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:SignatureHelpArgument,
) -> Result<Option<SignatureHelpResultDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_signature_help", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "sig_help")?;

	let position_dto = PositionDto { line_number:args.line_number_0_based, column:args.column_0_based };

	let effect = provide_signature_help_effect(
		target_uri,
		language_id,
		position_dto,
		args.context_dto,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "sig_help_effect"))
}

#[derive(Deserialize, Debug)]
struct RequestReferencesArgument {
	uri_string:String,

	line_number_0_based:u32,

	column_0_based:u32,

	context_dto:Value, // IReferenceContext as Value
	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_request_references(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:RequestReferencesArgument,
) -> Result<Option<Vec<LocationLinkDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_references", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "references")?;

	let position_dto = PositionDto { line_number:args.line_number_0_based, column:args.column_0_based };

	let effect = provide_references_effect(
		target_uri,
		language_id,
		position_dto,
		args.context_dto,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "references_effect"))
}

#[derive(Deserialize, Debug)]
struct PrepareRenameArgument {
	uri_string:String,

	line_number_0_based:u32,

	column_0_based:u32,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_prepare_rename(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:PrepareRenameArgument,
) -> Result<Option<Value>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_prepare_rename", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "prepare_rename")?;

	let position_dto = PositionDto { line_number:args.line_number_0_based, column:args.column_0_based };

	let effect = prepare_rename_effect(target_uri, language_id, position_dto, args.cancellation_token_id_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "prepare_rename_effect"))
}

#[derive(Deserialize, Debug)]
struct ProvideRenameEditsArgument {
	uri_string:String,

	line_number_0_based:u32,

	column_0_based:u32,

	new_name:String,

	cancellation_token_id_val:Option<Value>,
}

#[command]
pub async fn mountain_provide_rename_edits(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:ProvideRenameEditsArgument,
) -> Result<Option<WorkspaceEditDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_provide_rename_edits", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "rename_edits")?;

	let position_dto = PositionDto { line_number:args.line_number_0_based, column:args.column_0_based };

	let effect = provide_rename_edits_effect(
		target_uri,
		language_id,
		position_dto,
		args.new_name,
		args.cancellation_token_id_val,
	);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "rename_edits_effect"))
}

#[command]
pub async fn mountain_apply_workspace_edit(
	runtime:State<'_, Arc<AppRuntime>>,

	edit_dto:WorkspaceEditDto,
) -> Result<bool, String> {
	info!("[Track Command] mountain_apply_workspace_edit: {} edits.", edit_dto.edits.len());

	trace!("[Track Command] WorkspaceEdit DTO: {:?}", edit_dto);

	let effect = apply_workspace_edit_effect(edit_dto);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "apply_workspace_edit_effect"))
}

#[derive(Deserialize, Debug)]
struct FormattingArgument {
	uri_string:String,

	options_dto:FormattingOptionsDto,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_document_formatting(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:FormattingArgument,
) -> Result<Option<Vec<TextEditDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_document_formatting", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "doc_formatting")?;

	let effect = provide_document_formatting_edits_effect(target_uri, language_id, args.options_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "doc_fmt_effect"))
}

#[derive(Deserialize, Debug)]
struct PositionalArgument {
	uri_string:String,

	line:u32,      // Renamed from line_number_0_based for consistency with linked_editing
	character:u32, // Renamed from column_0_based
	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_document_highlights(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:PositionalArgument,
) -> Result<Option<Vec<DocumentHighlightDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_document_highlights", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "doc_highlights")?;

	let pos = PositionDto { line_number:args.line, column:args.character };

	let effect = provide_document_highlights_effect(target_uri, language_id, pos, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "doc_highlights_effect"))
}

#[derive(Deserialize, Debug)]
struct DocumentArgument {
	uri_string:String,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_document_links(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:DocumentArgument,
) -> Result<Option<LinksListDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_document_links", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "doc_links")?;

	let effect = provide_document_links_effect(target_uri, language_id, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "doc_links_effect"))
}

#[derive(Deserialize, Debug)]
struct ResolveLinkArgument {
	// list_cache_id: u32, // The effect takes list_cache_id, but VS Code resolveLink takes the link itself
	link_dto_val:Value, // LinkDto as Value, which should contain data for resolve
	token_val:Option<Value>,
}

#[command]
pub async fn mountain_resolve_document_link(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ResolveLinkArgument,
) -> Result<Option<LinkDto>, String> {
	// The effect now takes link_to_resolve_dto (as Value) directly
	let effect = resolve_document_link_effect(args.link_dto_val, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "resolve_link_effect"))
}

#[derive(Deserialize, Debug)]
struct FoldingRangesArgument {
	uri_string:String,

	context_dto:Value, // FoldingContext DTO as Value
	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_folding_ranges(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:FoldingRangesArgument,
) -> Result<Option<Vec<FoldingRangeDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_folding_ranges", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "folding_ranges")?;

	let effect = provide_folding_ranges_effect(target_uri, language_id, args.context_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "folding_ranges_effect"))
}

#[derive(Deserialize, Debug)]
struct SelectionRangesArgument {
	uri_string:String,

	positions_dto:Vec<PositionDto>,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_selection_ranges(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:SelectionRangesArgument,
) -> Result<Option<Vec<SelectionRangeDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_selection_ranges", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "sel_ranges")?;

	let effect = provide_selection_ranges_effect(target_uri, language_id, args.positions_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "sel_ranges_effect"))
}

#[derive(Deserialize, Debug)]
struct LinkedEditingArgument {
	uri_string:String,

	line:u32,

	character:u32,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_linked_editing_ranges(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:LinkedEditingArgument,
) -> Result<Option<LinkedEditingRangesDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string(
			"mountain_request_linked_editing_ranges",
			"uri_string",
			&e.to_string(),
			None,
		)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "linked_edit")?;

	let pos = PositionDto { line_number:args.line, column:args.character };

	let effect = provide_linked_editing_ranges_effect(target_uri, language_id, pos, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "linked_edit_effect"))
}

#[derive(Deserialize, Debug)]
struct SemanticTokensArgument {
	uri_string:String,

	previous_result_id:Option<String>,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_document_semantic_tokens(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:SemanticTokensArgument,
) -> Result<Option<SemanticTokensDto>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string(
			"mountain_request_document_semantic_tokens",
			"uri_string",
			&e.to_string(),
			None,
		)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "doc_sem_tok")?;

	let effect =
		provide_document_semantic_tokens_effect(target_uri, language_id, args.previous_result_id, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "doc_sem_tok_effect"))
}

#[derive(Deserialize, Debug)]
struct SemanticTokensEditsArgument {
	uri_string:String,

	previous_result_id:String,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_document_semantic_tokens_edits(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:SemanticTokensEditsArgument,
) -> Result<Option<Value>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string(
			"mountain_request_document_semantic_tokens_edits",
			"uri_string",
			&e.to_string(),
			None,
		)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "doc_sem_tok_edits")?;

	let effect =
		provide_document_semantic_tokens_edits_effect(target_uri, language_id, args.previous_result_id, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "doc_sem_tok_edits_effect"))
}

// TODO: Command for range semantic tokens

#[derive(Deserialize, Debug)]
struct PrepareHierarchyArgument {
	uri_string:String,

	line:u32,

	character:u32,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_prepare_call_hierarchy(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:PrepareHierarchyArgument,
) -> Result<Option<Vec<HierarchyItemDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_prepare_call_hierarchy", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "prep_call_hier")?;

	let pos = PositionDto { line_number:args.line, column:args.character };

	let effect = prepare_call_hierarchy_effect(target_uri, language_id, pos, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "prep_call_hier_effect"))
}

#[derive(Deserialize, Debug)]
struct ProvideHierarchyDetailArgument {
	item_dto:HierarchyItemDto, // Contains _sessionId, _itemId from previous step
	token_val:Option<Value>,
}

#[command]
pub async fn mountain_provide_call_hierarchy_incoming(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ProvideHierarchyDetailArgument,
) -> Result<Option<Vec<IncomingCallDto>>, String> {
	let effect = provide_call_hierarchy_incoming_calls_effect(args.item_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "call_hier_in_effect"))
}

#[command]
pub async fn mountain_provide_call_hierarchy_outgoing(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ProvideHierarchyDetailArgument,
) -> Result<Option<Vec<OutgoingCallDto>>, String> {
	let effect = provide_call_hierarchy_outgoing_calls_effect(args.item_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "call_hier_out_effect"))
}

#[command]
pub async fn mountain_prepare_type_hierarchy(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:PrepareHierarchyArgument,
) -> Result<Option<Vec<HierarchyItemDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_prepare_type_hierarchy", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "prep_type_hier")?;

	let pos = PositionDto { line_number:args.line, column:args.character };

	let effect = prepare_type_hierarchy_effect(target_uri, language_id, pos, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "prep_type_hier_effect"))
}

#[command]
pub async fn mountain_provide_type_hierarchy_supertypes(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ProvideHierarchyDetailArgument,
) -> Result<Option<Vec<HierarchyItemDto>>, String> {
	let effect = provide_type_hierarchy_supertypes_effect(args.item_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "type_hier_super_effect"))
}

#[command]
pub async fn mountain_provide_type_hierarchy_subtypes(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ProvideHierarchyDetailArgument,
) -> Result<Option<Vec<HierarchyItemDto>>, String> {
	let effect = provide_type_hierarchy_subtypes_effect(args.item_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "type_hier_sub_effect"))
}

#[derive(Deserialize, Debug)]
struct RequestInlayHintsArgument {
	uri_string:String,

	start_line:u32,

	start_char:u32,

	end_line:u32,

	end_char:u32,

	token_val:Option<Value>,
}

#[command]
pub async fn mountain_request_inlay_hints(
	app_handle:AppHandle<Wry>,

	runtime:State<'_, Arc<AppRuntime>>,

	args:RequestInlayHintsArgument,
) -> Result<Option<Vec<InlayHintDto>>, String> {
	let target_uri = Url::parse(&args.uri_string).map_err(|e| {
		error_utils::rpc_param_error_string("mountain_request_inlay_hints", "uri_string", &e.to_string(), None)
	})?;

	let language_id = get_language_id_for_uri(&app_handle, &target_uri, "inlay_hints")?;

	let range_dto = RangeDto {
		start_line_number:args.start_line,

		start_column:args.start_char,

		end_line_number:args.end_line,

		end_column:args.end_char,
	};

	let effect = provide_inlay_hints_effect(target_uri, language_id, range_dto, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "inlay_hints_effect"))
}

#[derive(Deserialize, Debug)]
struct ResolveInlayHintArgument {
	// provider_handle: u32, // This was present in one snippet but resolve_inlay_hint_effect takes hint_dto_val
	// directly
	hint_dto_to_resolve_val:Value, // InlayHintDto as Value
	token_val:Option<Value>,
}

#[command]
pub async fn mountain_resolve_inlay_hint(
	runtime:State<'_, Arc<AppRuntime>>,

	args:ResolveInlayHintArgument,
) -> Result<Option<InlayHintDto>, String> {
	let effect = resolve_inlay_hint_effect(args.hint_dto_to_resolve_val, args.token_val);

	runtime
		.run(effect)
		.await
		.map_err(|e| map_common_error_to_rpc_error_string(e, "resolve_inlay_hint_effect"))
}
