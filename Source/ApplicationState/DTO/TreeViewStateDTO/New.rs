//! `TreeViewStateDTO::New`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

pub fn Fn(
		ViewIdentifier:String,

		Provider:Option<Arc<dyn TreeViewProvider + Send + Sync>>,

		SideCarIdentifier:Option<String>,

		CanSelectMany:bool,

		HasHandleDrag:bool,

		HasHandleDrop:bool,
	) -> Result<Self, String> {
		// Validate view identifier length
		if ViewIdentifier.len() > MAX_VIEW_IDENTIFIER_LENGTH {
			return Err(format!(
				"View identifier exceeds maximum length of {} bytes",
				MAX_VIEW_IDENTIFIER_LENGTH
			));
		}

		// Validate sidecar identifier length
		if let Some(SideCarID) = &SideCarIdentifier {
			if SideCarID.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
				return Err(format!(
					"SideCar identifier exceeds maximum length of {} bytes",
					MAX_SIDECAR_IDENTIFIER_LENGTH
				));
			}
		}

		Ok(Self {
			ViewIdentifier,
			Provider,
			SideCarIdentifier,
			CanSelectMany,
			HasHandleDrag,
			HasHandleDrop,
			Message:None,
			Title:None,
			Description:None,
			Badge:None,
		})
	}
