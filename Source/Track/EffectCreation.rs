// File: Mountain/Source/Track/EffectCreation.rs
// Role: Central routing table for the application.
// Responsibilities:
//   - Map string-based command and RPC method names to their strongly-typed
//     effect constructors from the `Common` crate.
//   - Create a runnable, type-erased `MappedEffect` for each request.

//! # EffectCreation
//!
//! Contains the logic for creating `ActionEffect`s by mapping string-based
//! command and RPC method names to their strongly-typed effect constructors in
//! the `Common` crate. This is the central routing table of the application.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin, sync::Arc};

use Common::{
	self,
	Command::{ExecuteCommand::ExecuteCommand, GetAllCommands::GetAllCommands, RegisterCommand::RegisterCommand},
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationTarget::ConfigurationTarget,
		GetConfiguration::GetConfiguration,
	},
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Debug::DebugService::DebugService,
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Document::{SaveDocument::SaveDocument, SaveDocumentAs::SaveDocumentAs},
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{
		Delete::Delete,
		ReadDirectory::ReadDirectory,
		ReadFile::ReadFile,
		StatFile::StatFile,
		WriteFileBytes::WriteFileBytes,
	},
	Keybinding::KeybindingProvider::KeybindingProvider,
	LanguageFeature::{
		DTO::ProviderType::ProviderType,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
	Search::SearchProvider::SearchProvider,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
	Storage::{GetStorageItem::GetStorageItem, SetStorageItem::SetStorageItem, StorageProvider::StorageProvider},
	Terminal::TerminalProvider::TerminalProvider,
	TreeView::TreeViewProvider::TreeViewProvider,
	UserInterface::{ShowMessage::ShowMessage, ShowOpenDialog::ShowOpenDialog, ShowSaveDialog::ShowSaveDialog},
	WebView::WebViewProvider::WebViewProvider,
};
use serde_json::{Value, from_value, json};
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime,
};

/// A type alias for a boxed, runnable effect. This is the "type-erased" unit of
/// work that the dispatch logic can execute.
pub type MappedEffect =
	Box<dyn FnOnce(Arc<MountainRunTime>) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> + Send>;

/// A helper macro to reduce boilerplate when getting and deserializing
/// parameters from a JSON array.
macro_rules! Parameter {
	($Parameter:expr, $Current:expr, $Type:ty) => {
		from_value::<$Type>(
			$Parameter
				.get($Current)
				.cloned()
				.ok_or_else(|| format!("Missing parameter at index {}", $Current))?,
		)
		.map_err(|Error| format!("Invalid parameter at index {}: {}", $Current, Error))
	};
}

/// A helper that takes a specific `ActionEffect`, boxes it, and returns a
/// closure that can be run by the dispatcher. This is used for effects that
/// follow the declarative pattern.
fn Map<C, O, E>(Effect:ActionEffect<Arc<C>, E, O>) -> MappedEffect
where
	C: ?Sized + Send + Sync + 'static,
	O: serde::Serialize + Send + Sync + 'static,
	E: Into<CommonError> + From<CommonError> + Send + Sync + 'static,
	MountainEnvironment: Requires<C>, {
	Box::new(move |RunTime:Arc<MountainRunTime>| {
		Box::pin(async move {
			let Result = RunTime.Run(Effect).await;
			match Result {
				Ok(Output) => serde_json::to_value(Output).map_err(|e| format!("Serialization failed: {}", e)),
				Err(Error) => {
					let CommonError:CommonError = Error.into();
					Err(CommonError.to_string())
				},
			}
		})
	})
}

/// Creates an `ActionEffect` or a direct provider call for a request from any
/// source (frontend or sidecar). This function is the primary router for the
/// entire backend application logic.
pub fn CreateEffectForRequest<R:Runtime>(
	_ApplicationHandle:&AppHandle<R>,
	Method:&str,
	Parameters:Value,
) -> Result<MappedEffect, String> {
	let ParametersArray = Parameters
		.as_array()
		.ok_or_else(|| format!("Parameters for '{}' must be an array.", Method))?;

	// --- Direct Provider Calls (for performance or simplicity) ---
	// These bypass the `box_effect` helper for direct invocation.
	match Method {
		// Configuration
		"Configuration.Inspect" => {
			let Key = Parameter!(ParametersArray, 0, String)?;
			let Overrides = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn ConfigurationInspector> = runtime.Environment.Require();
					let result = provider
						.InspectConfigurationValue(Key, from_value(Overrides).unwrap_or_default())
						.await
						.map_err(|e| e.to_string())?;
					Ok(serde_json::to_value(result).unwrap_or(Value::Null))
				})
			}));
		},
		"Configuration.Update" => {
			let Key = Parameter!(ParametersArray, 0, String)?;
			let ValueToSet = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			let Target = Parameter!(ParametersArray, 2, ConfigurationTarget)?;
			let Overrides = ParametersArray.get(3).cloned().unwrap_or(Value::Null);
			let ScopeToLang = Parameter!(ParametersArray, 4, Option<bool>)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();
					provider
						.UpdateConfigurationValue(
							Key,
							ValueToSet,
							Target,
							from_value(Overrides).unwrap_or_default(),
							ScopeToLang,
						)
						.await
						.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Custom Editor
		"$resolveCustomEditor" => {
			let ViewType = Parameter!(ParametersArray, 0, String)?;
			let ResourceURI = Parameter!(ParametersArray, 1, Url)?;
			let WebViewPanelHandle = Parameter!(ParametersArray, 2, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn CustomEditorProvider> = runtime.Environment.Require();
					provider
						.ResolveCustomEditor(ViewType, ResourceURI, WebViewPanelHandle)
						.await
						.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Debug
		"Debug.Start" => {
			let folder_uri_val = ParametersArray.get(0).cloned().unwrap_or(Value::Null);
			let config = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			let folder_uri = if folder_uri_val.is_null() {
				None
			} else {
				Some(from_value(folder_uri_val).map_err(|e| e.to_string())?)
			};
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DebugService> = runtime.Environment.Require();
					let session_id = provider.StartDebugging(folder_uri, config).await.map_err(|e| e.to_string())?;
					Ok(json!(session_id))
				})
			}));
		},
		"Debug.RegisterConfigurationProvider" => {
			let dbg_type = Parameter!(ParametersArray, 0, String)?;
			let handle = Parameter!(ParametersArray, 1, u32)?;
			let sidecar_id = Parameter!(ParametersArray, 2, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DebugService> = runtime.Environment.Require();
					provider
						.RegisterDebugConfigurationProvider(dbg_type, handle, sidecar_id)
						.await
						.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Diagnostic
		"Diagnostic.Set" => {
			let Owner = Parameter!(ParametersArray, 0, String)?;
			let Entries = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DiagnosticManager> = runtime.Environment.Require();
					provider.SetDiagnostics(Owner, Entries).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"Diagnostic.Clear" => {
			let Owner = Parameter!(ParametersArray, 0, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DiagnosticManager> = runtime.Environment.Require();
					provider.ClearDiagnostics(Owner).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Keybinding
		"Keybinding.GetResolved" => {
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn KeybindingProvider> = runtime.Environment.Require();
					provider.GetResolvedKeybinding().await.map_err(|e| e.to_string())
				})
			}));
		},
		// Language Features
		"$languageFeatures:registerProvider" => {
			let SidecarID = Parameter!(ParametersArray, 0, String)?;
			let ProviderType = Parameter!(ParametersArray, 1, ProviderType)?;
			let Selector = ParametersArray.get(2).cloned().unwrap_or(Value::Null);
			let ExtensionID = ParametersArray.get(3).cloned().unwrap_or(Value::Null);
			let Options = ParametersArray.get(4).cloned();
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn LanguageFeatureProviderRegistry> = runtime.Environment.Require();
					let handle = provider
						.RegisterProvider(SidecarID, ProviderType, Selector, ExtensionID, Options)
						.await
						.map_err(|e| e.to_string())?;
					Ok(json!(handle))
				})
			}));
		},
		"$languageFeatures:unregisterProvider" => {
			let Handle = Parameter!(ParametersArray, 0, u32)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn LanguageFeatureProviderRegistry> = runtime.Environment.Require();
					provider.UnregisterProvider(Handle).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Search
		"Search.TextSearch" => {
			let Query = ParametersArray.get(0).cloned().unwrap_or(Value::Null);
			let Options = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SearchProvider> = runtime.Environment.Require();
					let result = provider.TextSearch(Query, Options).await.map_err(|e| e.to_string())?;
					Ok(result)
				})
			}));
		},
		// SourceControlManagement
		"$scm:createSourceControl" => {
			let DTO = ParametersArray.get(0).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();
					let handle = provider.CreateSourceControl(DTO).await.map_err(|e| e.to_string())?;
					Ok(json!(handle))
				})
			}));
		},
		"$scm:updateSourceControl" => {
			let handle = Parameter!(ParametersArray, 0, u32)?;
			let dto = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();
					provider.UpdateSourceControl(handle, dto).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"$scm:updateGroup" => {
			let handle = Parameter!(ParametersArray, 0, u32)?;
			let dto = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();
					provider
						.UpdateSourceControlGroup(handle, dto)
						.await
						.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"$scm:registerInputBox" => {
			let handle = Parameter!(ParametersArray, 0, u32)?;
			let dto = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();
					provider.RegisterInputBox(handle, dto).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Status Bar
		"$statusBar:set" => {
			let EntryDTO = Parameter!(ParametersArray, 0, StatusBarEntryDTO)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();
					provider.SetStatusBarEntry(EntryDTO).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"$statusBar:dispose" => {
			let EntryID = Parameter!(ParametersArray, 0, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();
					provider.DisposeStatusBarEntry(EntryID).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"$setStatusBarMessage" => {
			let ID = Parameter!(ParametersArray, 0, String)?;
			let Text = Parameter!(ParametersArray, 1, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();
					provider.SetStatusBarMessage(ID, Text).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"$disposeStatusBarMessage" => {
			let ID = Parameter!(ParametersArray, 0, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();
					provider.DisposeStatusBarMessage(ID).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Storage (Batch)
		"$storage:getAll" => {
			let IsGlobal = Parameter!(ParametersArray, 0, bool)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();
					provider.GetAllStorage(IsGlobal).await.map_err(|e| e.to_string())
				})
			}));
		},
		"$storage:setAll" => {
			let IsGlobal = Parameter!(ParametersArray, 0, bool)?;
			let State = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();
					provider.SetAllStorage(IsGlobal, State).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Terminal
		"$terminal:create" => {
			let OptionsValue = ParametersArray.get(0).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();
					let result = provider.CreateTerminal(OptionsValue).await.map_err(|e| e.to_string())?;
					Ok(result)
				})
			}));
		},
		"$terminal:sendText" => {
			let TerminalId = Parameter!(ParametersArray, 0, u64)?;
			let Text = Parameter!(ParametersArray, 1, String)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();
					provider.SendTextToTerminal(TerminalId, Text).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		"$terminal:dispose" => {
			let TerminalId = Parameter!(ParametersArray, 0, u64)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();
					provider.DisposeTerminal(TerminalId).await.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// Tree View
		"$tree:register" => {
			let ID = Parameter!(ParametersArray, 0, String)?;
			let Options = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TreeViewProvider> = runtime.Environment.Require();
					provider
						.RegisterTreeDataProvider(ID, Options)
						.await
						.map_err(|e| e.to_string())?;
					Ok(Value::Null)
				})
			}));
		},
		// WebView
		"$webview:create" => {
			let ExtData = Parameter!(ParametersArray, 0, Value)?;
			let ViewType = Parameter!(ParametersArray, 1, String)?;
			let Title = Parameter!(ParametersArray, 2, String)?;
			let ShowOpts = Parameter!(ParametersArray, 3, Value)?;
			let PanelOpts = Parameter!(ParametersArray, 4, Value)?;
			let ContentOpts = Parameter!(ParametersArray, 5, Value)?;
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn WebViewProvider> = runtime.Environment.Require();
					let handle = provider
						.CreateWebViewPanel(ExtData, ViewType, Title, ShowOpts, PanelOpts, ContentOpts)
						.await
						.map_err(|e| e.to_string())?;
					Ok(json!(handle))
				})
			}));
		},
		_ => {}, // Fall through to effect-based handlers
	}

	// --- ActionEffect-based Handlers ---
	let effect = match Method {
		// Command
		"Command.Execute" => {
			let ID = Parameter!(ParametersArray, 0, String)?;
			let Args = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			Map(ExecuteCommand(ID, Args))
		},
		"Command.GetAll" => Map(GetAllCommands()),
		"Command.Register" => {
			let SidecarID = Parameter!(ParametersArray, 0, String)?;
			let CommandID = Parameter!(ParametersArray, 1, String)?;
			Map(RegisterCommand(SidecarID, CommandID))
		},
		// Configuration
		"Configuration.Get" => {
			let Section = Parameter!(ParametersArray, 0, Option<String>)?;
			let Overrides = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			Map(GetConfiguration(Section, Overrides))
		},
		// Document
		"Document.Save" => {
			let uri_str = Parameter!(ParametersArray, 0, String)?;
			let uri = Url::parse(&uri_str).map_err(|e| format!("Invalid URI parameter: {}", e))?;
			Map(SaveDocument(uri))
		},
		"Document.SaveAs" => {
			let original_uri_str = Parameter!(ParametersArray, 0, String)?;
			let original_uri = Url::parse(&original_uri_str).map_err(|e| format!("Invalid URI parameter: {}", e))?;
			Map(SaveDocumentAs(original_uri, None))
		},
		// FileSystem
		"FileSystem.ReadFile" => {
			let Path = Parameter!(ParametersArray, 0, _)?;
			Map(ReadFile(Path))
		},
		"FileSystem.StatFile" => {
			let Path = Parameter!(ParametersArray, 0, _)?;
			Map(StatFile(Path))
		},
		"FileSystem.ReadDirectory" => {
			let Path = Parameter!(ParametersArray, 0, _)?;
			Map(ReadDirectory(Path))
		},
		"FileSystem.WriteFile" => {
			let Path = Parameter!(ParametersArray, 0, _)?;
			let Content = Parameter!(ParametersArray, 1, Vec<u8>)?;
			let Create = Parameter!(ParametersArray, 2, bool)?;
			let Overwrite = Parameter!(ParametersArray, 3, bool)?;
			Map(WriteFileBytes(Path, Content, Create, Overwrite))
		},
		"FileSystem.Delete" => {
			let Path = Parameter!(ParametersArray, 0, _)?;
			let Recursive = Parameter!(ParametersArray, 1, bool)?;
			let UseTrash = Parameter!(ParametersArray, 2, bool)?;
			Map(Delete(Path, Recursive, UseTrash))
		},
		// Storage (Legacy)
		"Storage.Get" => {
			let TargetObject = ParametersArray.get(0).cloned().unwrap_or(Value::Null);
			Map(GetStorageItem(TargetObject))
		},
		"Storage.Set" => {
			let TargetObject = ParametersArray.get(0).cloned().unwrap_or(Value::Null);
			let ValueToSet = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			Map(SetStorageItem(TargetObject, ValueToSet))
		},
		// UserInterface
		"UserInterface.ShowMessage" => {
			let Severity = Parameter!(ParametersArray, 0, _)?;
			let Message = Parameter!(ParametersArray, 1, String)?;
			let Options = ParametersArray.get(2).cloned().unwrap_or(Value::Null);
			Map(ShowMessage(Severity, Message, Options))
		},
		"UserInterface.ShowOpenDialog" => {
			let Options = Parameter!(ParametersArray, 0, _)?;
			Map(ShowOpenDialog(Options))
		},
		"UserInterface.ShowSaveDialog" => {
			let Options = Parameter!(ParametersArray, 0, _)?;
			Map(ShowSaveDialog(Options))
		},
		_ => return Err(format!("No mapping found for method: {}", Method)),
	};

	Ok(effect)
}
