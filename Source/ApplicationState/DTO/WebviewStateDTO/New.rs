//! `WebviewStateDTO::New`

use super::Struct;
use CommonLibrary::Webview::DTO::WebviewContentOptionsDTO::WebviewContentOptionsDTO;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(
		Handle:String,

		ViewType:String,

		Title:String,

		ContentOptions:WebviewContentOptionsDTO,

		PanelOptions:Value,

		SideCarIdentifier:String,

		ExtensionIdentifier:String,
	) -> Result<Self, String> {
		// Validate handle length
		if Handle.len() > MAX_HANDLE_LENGTH {
			return Err(format!("Handle exceeds maximum length of {} bytes", MAX_HANDLE_LENGTH));
		}

		// Validate view type length
		if ViewType.len() > MAX_VIEW_TYPE_LENGTH {
			return Err(format!("ViewType exceeds maximum length of {} bytes", MAX_VIEW_TYPE_LENGTH));
		}

		// Validate title length
		if Title.len() > MAX_TITLE_LENGTH {
			return Err(format!("Title exceeds maximum length of {} bytes", MAX_TITLE_LENGTH));
		}

		// Validate sidecar identifier length
		if SideCarIdentifier.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
			return Err(format!(
				"SideCar identifier exceeds maximum length of {} bytes",
				MAX_SIDECAR_IDENTIFIER_LENGTH
			));
		}

		// Validate extension identifier length
		if ExtensionIdentifier.len() > MAX_EXTENSION_IDENTIFIER_LENGTH {
			return Err(format!(
				"Extension identifier exceeds maximum length of {} bytes",
				MAX_EXTENSION_IDENTIFIER_LENGTH
			));
		}

		Ok(Self {
			Handle,
			ViewType,
			Title,
			ContentOptions,
			PanelOptions,
			SideCarIdentifier,
			ExtensionIdentifier,
			IsActive:false,
			IsVisible:false,
		})
	}
