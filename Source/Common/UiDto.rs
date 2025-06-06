// File: Common/UiDto.rs
// Defines Data Transfer Objects (DTOs) for various UI elements like dialogs,
// quick picks, and input boxes, used for communication between the backend and
// frontend.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

/// Specifies the severity level for a message dialog.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageSeverity {
	Info,
	Warning,
	Error,
}

/// Defines options for a message dialog.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MessageOptions {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Title:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Modal:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Detail:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ItemList:Option<Vec<String>>,
}

/// Represents a filter for file dialogs.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct FileFilter {
	pub Name:String,
	#[serde(alias = "extensions")]
	pub ExtensionList:Vec<String>,
}

/// Common base options for file dialogs.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct DialogOptions {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Title:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "defaultPath")]
	pub DefaultPath:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "filters")]
	pub FilterList:Option<Vec<FileFilter>>,
}

/// Defines options for a file open dialog.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OpenDialogOptions {
	#[serde(flatten)]
	pub Base:DialogOptions,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Multiple:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Directory:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Recursive:Option<bool>,
}

/// Defines options for a file save dialog.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct SaveDialogOptions {
	#[serde(flatten)]
	pub Base:DialogOptions,
}

/// Represents a single item in a quick pick list.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct QuickPickItem {
	pub Label:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Description:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Detail:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Picked:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "alwaysShow")]
	pub AlwaysShow:Option<bool>,
}

/// Defines options for a quick pick UI element.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct QuickPickOptions {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Title:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "placeHolder")]
	pub PlaceHolder:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "canPickMany")]
	pub CanPickMany:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "ignoreFocusOut")]
	pub IgnoreFocusOut:Option<bool>,
}

/// Defines options for an input box UI element.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InputBoxOptions {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Title:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "placeHolder")]
	pub PlaceHolder:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Value:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Prompt:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Password:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "ignoreFocusOut")]
	pub IgnoreFocusOut:Option<bool>,
}
