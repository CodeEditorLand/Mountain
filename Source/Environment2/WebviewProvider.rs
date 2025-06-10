use std::sync::Arc;

use Common::{
	environment::Requires,
	error::CommonError,
	webview::{WebviewProvider, dto::*},
};
use async_trait::async_trait;
use serde_json::Value;

/// @module WebviewProvider (Environment)
/// @description Implements the `WebviewProvider` trait for
/// `MountainEnvironment` by delegating to the logic handlers in
/// `handlers::webview`.
use super::MountainEnvironment;
use crate::handlers::webview as WebviewHandler;

#[async_trait]
impl WebviewProvider for MountainEnvironment {
	async fn CreateWebviewPanel(
		&self,
		ExtensionData:WebviewExtensionDescriptionDto,
		ViewType:String,
		Title:String,
		ShowOptions:WebviewShowOptionsDto,
		PanelOptions:WebviewPanelOptionsDto,
		ContentOptions:WebviewContentOptionsDto,
		SerializeBuffers:bool,
	) -> Result<String, CommonError> {
		WebviewHandler::CreateWebviewPanelLogic(
			&self.AppHandle,
			ExtensionData,
			ViewType,
			Title,
			ShowOptions,
			PanelOptions,
			ContentOptions,
			SerializeBuffers,
		)
		.await
	}

	async fn DisposeWebview(&self, Handle:String) -> Result<(), CommonError> {
		WebviewHandler::DisposeWebviewLogic(&self.AppHandle, Handle).await
	}

	async fn RevealWebviewPanel(&self, Handle:String, ShowOptions:WebviewShowOptionsDto) -> Result<(), CommonError> {
		WebviewHandler::RevealWebviewPanelLogic(&self.AppHandle, Handle, ShowOptions).await
	}

	async fn SetWebviewTitle(&self, Handle:String, Title:String) -> Result<(), CommonError> {
		// This would delegate to a `SetWebviewTitleLogic` handler.
		// For now, we stub it as it's a simple UI update.
		Ok(())
	}

	async fn SetWebviewIconPath(&self, Handle:String, IconPath:Option<Value>) -> Result<(), CommonError> {
		// This would delegate to a `SetWebviewIconPathLogic` handler.
		Ok(())
	}

	async fn SetWebviewHtml(&self, Handle:String, Html:String) -> Result<(), CommonError> {
		WebviewHandler::SetWebviewHtmlLogic(&self.AppHandle, Handle, Html).await
	}

	async fn PostMessageToWebview(&self, Handle:String, Message:Value) -> Result<bool, CommonError> {
		WebviewHandler::PostMessageToWebviewLogic(&self.AppHandle, Handle, Message).await
	}
}

impl Requires<Arc<dyn WebviewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebviewProvider + Send + Sync> { Arc::new(self.clone()) }
}
