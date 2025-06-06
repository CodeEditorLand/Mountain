// ---------------------------------------------------------------------------------------------
// Mountain Environment - UI Provider 
// --------------------------------------------------------------------------------------------
// This module implements the `UiProvider` trait for `MountainEnvironment`.
// It handles UI interactions initiated by backend effects, such as showing
// messages, file dialogs, quick picks, and input boxes. These interactions
// are typically relayed to the Sky frontend for actual display and user input.
//
// For complex interactions, it uses an asynchronous request-response pattern
// with Sky via Tauri events and `AppState.pending_ui_requests`.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

use Land_Common::{
	environment::Requires,
	errors::CommonError,
	ui_effects::{
		DialogOptions,
		InputBoxOptions,
		MessageOptions,
		MessageSeverity,
		OpenDialogOptions,
		QuickPickItem,
		QuickPickOptions,
		SaveDialogOptions,
		UiProvider,
	},
};
use async_trait::async_trait;
use log::{debug, error, info, warn}; // Added debug
use serde::Serialize; // For UiRequestToSkyPayload
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime as TauriRuntime, Wry};
use tokio::{
	sync::oneshot as TokioOneshot,
	time::{Duration as TokioDuration, timeout as tokio_timeout},
};
use uuid::Uuid;

use crate::environment::{
	MountainEnvironment,
	utils::map_app_state_lock_error_to_common_error, // For AppState lock errors
};

/// Helper struct for serializing UiProvider request payloads sent via Tauri
/// events to Sky.
#[derive(Serialize, Clone, Debug)]
struct UiRequestToSkyPayload<T:Serialize + Clone> {
	request_id:String,
	// Payload specific to the UI request type (e.g., OpenDialogOptions, MessageOptions + message text)
	payload:T,
}

// --- UiProvider Implementation ---
#[async_trait]
impl UiProvider for MountainEnvironment {
	async fn show_message(
		&self,
		severity:MessageSeverity,
		message_text:String,
		options_json_val_opt:Option<Value>, // MessageOptions DTO as Value
	) -> Result<Option<String>, CommonError> {
		let severity_str = match severity {
			MessageSeverity::Info => "info",
			MessageSeverity::Warning => "warn",
			MessageSeverity::Error => "error",
		};
		info!(
			"[Env UiProv ShowMessage] Severity='{}', Message='{}...', OptionsIsSome={}",
			severity_str,
			message_text.chars().take(50).collect::<String>(),
			options_json_val_opt.is_some()
		);

		// Determine if simple dialog can be used (no buttons, not modal by default)
		let (use_simple_dialog, title_for_simple) = if let Some(opts_val) = &options_json_val_opt {
			// Attempt to deserialize to MessageOptions to check `items` and `modal`
			let deserialized_opts:Result<MessageOptions, _> = serde_json::from_value(opts_val.clone());
			match deserialized_opts {
				Ok(opts_dto) => {
					let items_empty = opts_dto.items.as_ref().map_or(true, Vec::is_empty);
					let not_modal = !opts_dto.modal.unwrap_or(false);
					(
						items_empty && not_modal,
						opts_dto
							.title
							.unwrap_or_else(|| format!("Land Editor - {}", severity_str.to_uppercase())),
					)
				},
				Err(_) => {
					// If options can't be parsed as MessageOptions, assume complex case or
					// fallback. For safety, treat as complex.
					warn!(
						"[Env UiProv ShowMessage] Could not parse options_json_val as MessageOptions for simple \
						 dialog check. Using Sky IPC."
					);
					(false, String::new()) // Title not used if false
				},
			}
		} else {
			// No options provided, definitely simple.
			(true, format!("Land Editor - {}", severity_str.to_uppercase()))
		};

		if use_simple_dialog {
			debug!("[Env UiProv ShowMessage] Using simple Tauri dialog (non-modal, no buttons).");
			let window_main = self
				.app_handle
				.get_webview_window("main")
				.ok_or_else(|| CommonError::UiInteraction("Main window not found for simple dialog.".to_string()))?;
			let message_clone_for_dialog = message_text.clone();

			tokio::task::spawn_blocking(move || {
				tauri::api::dialog::message(Some(&window_main), title_for_simple, message_clone_for_dialog);
			})
			.await
			.map_err(|e_join| {
				CommonError::UiInteraction(format!("Failed to spawn blocking task for simple dialog: {}", e_join))
			})?;
			return Ok(None); // Simple dialogs here don't return selections.
		}

		// Complex case: Modal or has buttons, use async request-response with Sky.
		debug!("[Env UiProv ShowMessage] Using Sky IPC for complex message dialog.");
		let request_id_str = Uuid::new_v4().to_string();
		let (response_sender_oneshot, response_receiver_oneshot) = TokioOneshot::channel();
		{
			let app_state = self.get_app_state();
			let mut pending_requests_guard = app_state
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;
			pending_requests_guard.insert(request_id_str.clone(), response_sender_oneshot);
		}

		let sky_event_payload_data = json!({
			"severity": severity_str,
			"message": message_text,
			"options": options_json_val_opt.unwrap_or(Value::Null)
		});
		let sky_event_full_payload =
			UiRequestToSkyPayload { request_id:request_id_str.clone(), payload:sky_event_payload_data };

		self.app_handle
			.emit("sky://ui/show-message-request", sky_event_full_payload)
			.map_err(|e_emit| {
				CommonError::UiInteraction(format!("Failed to emit 'sky://ui/show-message-request': {}", e_emit))
			})?;

		let ui_response_result = match tokio_timeout(TokioDuration::from_secs(300), response_receiver_oneshot).await {
			// 5 min timeout
			Ok(Ok(Ok(value_from_sky))) => {
				if value_from_sky.is_null() {
					Ok(None)
				} else if let Some(selected_item_title_str) = value_from_sky.as_str() {
					Ok(Some(selected_item_title_str.to_string()))
				} else {
					Err(CommonError::UiInteraction(
						"showMessage response from Sky was not string or null.".to_string(),
					))
				}
			},
			Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky), // Error reported by Sky's handler
			Ok(Err(_channel_closed_err)) => {
				Err(CommonError::UiInteraction(format!(
					"UiProvider showMessage (ReqID: {}): Response channel closed by Sky handler.",
					request_id_str
				)))
			},
			Err(_timeout_elapsed_err) => {
				warn!(
					"[Env UiProv ShowMessage] Timed out (ReqID: {}). Assuming dismissal.",
					request_id_str
				);
				Ok(None)
			},
		};

		if let Ok(mut guard) = self.get_app_state().pending_ui_requests.lock() {
			guard.remove(&request_id_str);
		} else {
			error!("[Env UiProv ShowMessage] Failed lock for cleanup (ReqID: {}).", request_id_str);
		}
		ui_response_result
	}

	async fn show_open_dialog(&self, options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		let request_id = Uuid::new_v4().to_string();
		info!("[Env UiProv ShowOpenDialog] ReqID: {}, options: {:?}", request_id, options);
		let (tx, rx) = TokioOneshot::channel();
		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}
		let event_payload =
			UiRequestToSkyPayload { request_id:request_id.clone(), payload:options.clone() /* DTO is Clone */ };
		self.app_handle
			.emit("sky://ui/show-open-dialog-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed emit show_open_dialog: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				if v.is_null() {
					Ok(None)
				} else if let Some(arr) = v.as_array() {
					arr.iter()
						.map(|p_val| {
							p_val.as_str().map(PathBuf::from).ok_or_else(|| {
								CommonError::UiInteraction("Invalid path string in open dialog response".into())
							})
						})
						.collect::<Result<Vec<_>, _>>()
						.map(Some)
				} else {
					Err(CommonError::UiInteraction("Open dialog response not array or null".into()))
				}
			},
			Ok(Ok(Err(e))) => Err(e),
			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"OpenDialog (ReqID: {}) channel closed.",
					request_id
				)))
			},
			Err(_) => {
				warn!("[Env UiProv ShowOpenDialog] Timed out (ReqID: {}).", request_id);
				Ok(None)
			},
		};
		if let Ok(mut guard) = self.get_app_state().pending_ui_requests.lock() {
			guard.remove(&request_id);
		}
		result
	}

	async fn show_save_dialog(&self, options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError> {
		let request_id = Uuid::new_v4().to_string();
		info!("[Env UiProv ShowSaveDialog] ReqID: {}, options: {:?}", request_id, options);
		let (tx, rx) = TokioOneshot::channel();
		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}
		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:options.clone() };
		self.app_handle
			.emit("sky://ui/show-save-dialog-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed emit show_save_dialog: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				if v.is_null() {
					Ok(None)
				} else if let Some(s) = v.as_str() {
					Ok(Some(PathBuf::from(s)))
				} else {
					Err(CommonError::UiInteraction("Save dialog response not string or null".into()))
				}
			},
			Ok(Ok(Err(e))) => Err(e),
			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"SaveDialog (ReqID: {}) channel closed.",
					request_id
				)))
			},
			Err(_) => {
				warn!("[Env UiProv ShowSaveDialog] Timed out (ReqID: {}).", request_id);
				Ok(None)
			},
		};
		if let Ok(mut guard) = self.get_app_state().pending_ui_requests.lock() {
			guard.remove(&request_id);
		}
		result
	}

	async fn show_quick_pick(
		&self,
		items:Vec<QuickPickItem>,
		options:Option<QuickPickOptions>,
	) -> Result<Option<Vec<String>>, CommonError> {
		let request_id = Uuid::new_v4().to_string();
		info!(
			"[Env UiProv ShowQuickPick] ReqID: {}, {} items, options: {:?}",
			request_id,
			items.len(),
			options
		);
		let (tx, rx) = TokioOneshot::channel();
		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}

		// QuickPickItem is already Serialize, so directly use it in the payload
		// construction.
		let payload_data = json!({ "items": items, "options": options });
		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:payload_data };

		self.app_handle
			.emit("sky://ui/show-quick-pick-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed emit show_quick_pick: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				if v.is_null() {
					Ok(None)
				} else if let Some(arr) = v.as_array() {
					// Expect array of strings for selection
					arr.iter()
						.map(|s_val| {
							s_val.as_str().map(String::from).ok_or_else(|| {
								CommonError::UiInteraction("Invalid string in quick pick response".into())
							})
						})
						.collect::<Result<Vec<_>, _>>()
						.map(Some)
				} else if let Some(s_single) = v.as_str() {
					// Handle single selection if `canPickMany` was false
					Ok(Some(vec![s_single.to_string()]))
				} else {
					Err(CommonError::UiInteraction(
						"Quick pick response not array, string or null".into(),
					))
				}
			},
			Ok(Ok(Err(e))) => Err(e),
			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"QuickPick (ReqID: {}) channel closed.",
					request_id
				)))
			},
			Err(_) => {
				warn!("[Env UiProv ShowQuickPick] Timed out (ReqID: {}).", request_id);
				Ok(None)
			},
		};
		if let Ok(mut guard) = self.get_app_state().pending_ui_requests.lock() {
			guard.remove(&request_id);
		}
		result
	}

	async fn show_input_box(&self, options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError> {
		let request_id = Uuid::new_v4().to_string();
		info!("[Env UiProv ShowInputBox] ReqID: {}, options: {:?}", request_id, options);
		let (tx, rx) = TokioOneshot::channel();
		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}
		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:options.clone() };
		self.app_handle
			.emit("sky://ui/show-input-box-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed emit show_input_box: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				if v.is_null() {
					Ok(None)
				} else if let Some(s) = v.as_str() {
					Ok(Some(s.to_string()))
				} else {
					Err(CommonError::UiInteraction("Input box response not string or null".into()))
				}
			},
			Ok(Ok(Err(e))) => Err(e),
			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"InputBox (ReqID: {}) channel closed.",
					request_id
				)))
			},
			Err(_) => {
				warn!("[Env UiProv ShowInputBox] Timed out (ReqID: {}).", request_id);
				Ok(None)
			},
		};
		if let Ok(mut guard) = self.get_app_state().pending_ui_requests.lock() {
			guard.remove(&request_id);
		}
		result
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}
