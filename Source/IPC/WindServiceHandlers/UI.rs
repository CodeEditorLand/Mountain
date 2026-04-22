#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! UI-layer handlers: themes, decorations, keybindings, notifications,
//! progress, quickinput, workspaces, lifecycle, working copy.

use std::sync::Arc;

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::AppHandle;

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify,
	},
	dev_log,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

// ============================================================================
// Themes handlers
// ============================================================================

/// Return the active color theme metadata from ConfigurationProvider.
pub async fn handle_themes_get_active(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	};

	let ThemeId = runtime
		.Environment
		.GetConfigurationValue(Some("workbench.colorTheme".to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("themes:getActive failed: {}", Error))?;

	let Id = ThemeId.as_str().unwrap_or("Default Dark Modern").to_string();

	// Infer kind from id string
	let Kind = if Id.to_lowercase().contains("light") {
		"light"
	} else if Id.to_lowercase().contains("high contrast light") {
		"highContrastLight"
	} else if Id.to_lowercase().contains("high contrast") {
		"highContrast"
	} else {
		"dark"
	};

	Ok(json!({ "id": Id, "label": Id, "kind": Kind }))
}

/// Return installed theme extensions.
pub async fn handle_themes_list(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Themes = vec![
		json!({ "id": "Default Dark Modern", "label": "Default Dark Modern", "kind": "dark" }),
		json!({ "id": "Default Light Modern", "label": "Default Light Modern", "kind": "light" }),
		json!({ "id": "Default Dark+", "label": "Default Dark+", "kind": "dark" }),
		json!({ "id": "Default Light+", "label": "Default Light+", "kind": "light" }),
		json!({ "id": "High Contrast", "label": "High Contrast", "kind": "highContrast" }),
		json!({ "id": "High Contrast Light", "label": "High Contrast Light", "kind": "highContrastLight" }),
	];

	Ok(json!(Themes))
}

/// Set the active color theme by updating ConfigurationProvider.
pub async fn handle_themes_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
	};
	use tauri::Emitter;

	let ThemeId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("themes:set requires themeId as first argument".to_string())?
		.to_string();

	runtime
		.Environment
		.UpdateConfigurationValue(
			"workbench.colorTheme".to_string(),
			json!(ThemeId),
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|Error| format!("themes:set failed: {}", Error))?;

	let _ = runtime
		.Environment
		.ApplicationHandle
		.emit(SkyEvent::ThemeChange.AsStr(), json!({ "themeId": ThemeId }));

	Ok(Value::Null)
}

// ============================================================================
// Decorations handlers
// ============================================================================

/// Return the decoration (badge, tooltip, color) for a single URI.
pub async fn handle_decorations_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:get requires uri".to_string())?;
	let Decoration = runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri);
	Ok(Decoration.unwrap_or(Value::Null))
}

/// Return decorations for multiple URIs in a single round-trip.
pub async fn handle_decorations_get_many(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let Uris:Vec<String> = args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| Arr.iter().filter_map(|U| U.as_str().map(str::to_owned)).collect())
		.unwrap_or_default();

	let mut Result = serde_json::Map::new();
	for Uri in &Uris {
		if let Some(Decoration) = runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
			Result.insert(Uri.clone(), Decoration);
		}
	}
	Ok(Value::Object(Result))
}

/// Register or override the decoration for a URI.
pub async fn handle_decorations_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:set requires uri".to_string())?;
	let Decoration = args.get(1).cloned().unwrap_or(Value::Null);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Decorations
		.SetDecoration(Uri, Decoration);
	Ok(Value::Null)
}

/// Remove the decoration for a URI.
pub async fn handle_decorations_clear(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:clear requires uri".to_string())?;
	runtime.Environment.ApplicationState.Feature.Decorations.ClearDecoration(Uri);
	Ok(Value::Null)
}

// ============================================================================
// Keybinding handlers
// ============================================================================

/// Register a dynamic keybinding in Mountain's keybinding registry.
pub async fn handle_keybinding_add(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires commandId".to_string())?
		.to_owned();
	let KeyExpression = args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires keybinding".to_string())?
		.to_owned();
	let When = args.get(2).and_then(|V| V.as_str()).map(str::to_owned);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.AddKeybinding(CommandId, KeyExpression, When);
	Ok(Value::Null)
}

/// Remove all dynamic keybindings for a command.
pub async fn handle_keybinding_remove(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:remove requires commandId".to_string())?;
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybinding(CommandId);
	Ok(Value::Null)
}

/// Look up the keybinding string for a command.
pub async fn handle_keybinding_lookup(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:lookup requires commandId".to_string())?;
	let Binding = runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.LookupKeybinding(CommandId);
	Ok(Binding.map(|B| json!(B)).unwrap_or(Value::Null))
}

/// Return all registered dynamic keybindings.
pub async fn handle_keybinding_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = runtime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();
	Ok(json!(All))
}

// ============================================================================
// Notification handlers
// ============================================================================

/// Show a notification message - emits sky://notification/show for Sky to render.
pub async fn handle_notification_show(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Message = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Severity = args.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();
	let Actions = args.get(2).cloned().unwrap_or(json!([]));

	let Id = format!(
		"notification-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		SkyEvent::NotificationShow.AsStr(),
		json!({
			"id": Id,
			"message": Message,
			"severity": Severity,
			"actions": Actions,
		}),
	);

	Ok(json!(Id))
}

/// Begin a progress notification - emits sky://notification/progress-begin.
pub async fn handle_notification_show_progress(
	app_handle:tauri::AppHandle,
	args:Vec<Value>,
) -> Result<Value, String> {
	use tauri::Emitter;

	let Title = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		SkyEvent::NotificationProgressBegin.AsStr(),
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Update an in-progress notification progress bar.
pub async fn handle_notification_update_progress(
	app_handle:tauri::AppHandle,
	args:Vec<Value>,
) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		SkyEvent::NotificationProgressUpdate.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress notification.
pub async fn handle_notification_end_progress(
	app_handle:tauri::AppHandle,
	args:Vec<Value>,
) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(SkyEvent::NotificationProgressEnd.AsStr(), json!({ "id": Id }));

	Ok(Value::Null)
}

// ============================================================================
// Progress handlers
// ============================================================================

/// Begin a window-level or status-bar progress indicator.
pub async fn handle_progress_begin(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Location = args.first().and_then(|V| V.as_str()).unwrap_or("notification").to_string();
	let Title = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		SkyEvent::ProgressBegin.AsStr(),
		json!({
			"id": Id,
			"location": Location,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Report incremental progress on an active indicator.
pub async fn handle_progress_report(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		SkyEvent::ProgressReport.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress indicator.
pub async fn handle_progress_end(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(SkyEvent::ProgressEnd.AsStr(), json!({ "id": Id }));

	Ok(Value::Null)
}

// ============================================================================
// QuickInput handlers
// ============================================================================

/// Show a quick-pick dialog. Routes through UserInterfaceProvider.
pub async fn handle_quick_input_show_quick_pick(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::{QuickPickItemDTO::QuickPickItemDTO, QuickPickOptionsDTO::QuickPickOptionsDTO},
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Items:Vec<QuickPickItemDTO> = args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| {
			Arr.iter()
				.filter_map(|Item| {
					let Label = Item.get("label").and_then(|L| L.as_str()).unwrap_or("").to_string();
					let Description = Item.get("description").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Detail = Item.get("detail").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Picked = Item.get("picked").and_then(|P| P.as_bool()).unwrap_or(false);
					Some(QuickPickItemDTO { Label, Description, Detail, Picked:Some(Picked), AlwaysShow:Some(false) })
				})
				.collect()
		})
		.unwrap_or_default();

	let Options = QuickPickOptionsDTO {
		PlaceHolder:args
			.get(1)
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		CanPickMany:Some(
			args.get(1)
				.and_then(|V| V.get("canPickMany"))
				.and_then(|B| B.as_bool())
				.unwrap_or(false),
		),
		Title:args
			.get(1)
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		..Default::default()
	};

	let Result = runtime
		.Environment
		.ShowQuickPick(Items, Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showQuickPick failed: {}", Error))?;

	match Result {
		Some(Labels) => Ok(Labels.into_iter().next().map(|S| json!(S)).unwrap_or(Value::Null)),
		None => Ok(Value::Null),
	}
}

/// Show an input box dialog. Routes through UserInterfaceProvider.
pub async fn handle_quick_input_show_input_box(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Opts = args.first();
	let Options = InputBoxOptionsDTO {
		Prompt:Opts
			.and_then(|V| V.get("prompt"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		PlaceHolder:Opts
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),
		Value:Opts
			.and_then(|V| V.get("value"))
			.and_then(|V| V.as_str())
			.map(|S| S.to_string()),
		Title:Opts
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		IgnoreFocusOut:None,
	};

	let Result = runtime
		.Environment
		.ShowInputBox(Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showInputBox failed: {}", Error))?;

	Ok(Result.map(|S| json!(S)).unwrap_or(Value::Null))
}

// ============================================================================
// Workspaces handlers
// ============================================================================

/// Return the current workspace folders.
pub async fn handle_workspaces_get_folders(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let Folders = Workspace.GetWorkspaceFolders();

	let FolderList:Vec<Value> = Folders
		.iter()
		.enumerate()
		.map(|(Index, Folder)| {
			json!({
				"uri": Folder.URI.to_string(),
				"name": Folder.Name,
				"index": Index,
			})
		})
		.collect();

	Ok(json!(FolderList))
}

/// Add a workspace folder.
pub async fn handle_workspaces_add_folder(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	use url::Url;

	let UriStr = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:addFolder requires uri as first argument".to_string())?
		.to_string();

	let Name = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	let Index = Folders.len();
	let URI = Url::parse(&UriStr).map_err(|E| format!("workspaces:addFolder invalid URI: {}", E))?;
	if let Ok(Folder) = WorkspaceFolderStateDTO::New(URI, Name, Index) {
		Folders.push(Folder);
		UpdateWorkspaceFoldersAndNotify(Workspace, Folders);
	}

	Ok(Value::Null)
}

/// Remove a workspace folder by URI.
pub async fn handle_workspaces_remove_folder(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let UriStr = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:removeFolder requires uri as first argument".to_string())?
		.to_string();

	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	Folders.retain(|F| F.URI.to_string() != UriStr);
	for (I, F) in Folders.iter_mut().enumerate() {
		F.Index = I;
	}
	UpdateWorkspaceFoldersAndNotify(Workspace, Folders);

	Ok(Value::Null)
}

/// Return the workspace name (basename of root folder, or None if untitled).
pub async fn handle_workspaces_get_name(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Name = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.GetDisplayName());

	Ok(Name.map(|N| json!(N)).unwrap_or(Value::Null))
}

// ============================================================================
// Lifecycle handlers
// ============================================================================

/// Return the current application lifecycle phase (1-4).
pub async fn handle_lifecycle_get_phase(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Phase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	Ok(json!(Phase))
}

/// Wait (poll) until the application reaches at least the requested phase.
pub async fn handle_lifecycle_when_phase(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let RequestedPhase = args.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;
	let CurrentPhase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	if CurrentPhase >= RequestedPhase {
		return Ok(Value::Null);
	}
	// Simple poll with short sleep - production should use a channel/notify
	let mut Retries = 0u8;
	loop {
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		let Phase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
		if Phase >= RequestedPhase || Retries >= 50 {
			break;
		}
		Retries += 1;
	}
	Ok(Value::Null)
}

/// Initiate a graceful application shutdown via Tauri.
pub async fn handle_lifecycle_request_shutdown(app_handle:AppHandle) -> Result<Value, String> {
	app_handle.exit(0);
	Ok(Value::Null)
}

// ============================================================================
// WorkingCopy handlers
// ============================================================================

/// Check whether a URI has unsaved changes.
pub async fn handle_working_copy_is_dirty(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;
	let IsDirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);
	Ok(json!(IsDirty))
}

/// Mark a URI as dirty (unsaved) or clean.
pub async fn handle_working_copy_set_dirty(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;
	let Dirty = args.get(1).and_then(|V| V.as_bool()).unwrap_or(true);
	runtime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);
	Ok(Value::Null)
}

/// Return all URIs that currently have unsaved changes.
pub async fn handle_working_copy_get_all_dirty(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();
	Ok(json!(Dirty))
}

/// Return the count of resources with unsaved changes.
pub async fn handle_working_copy_get_dirty_count(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();
	Ok(json!(Count))
}
