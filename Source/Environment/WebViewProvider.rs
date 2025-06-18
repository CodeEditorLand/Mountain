// @module WebViewProvider (Environment)
// @description Implements the `WebViewProvider` trait for
// `MountainEnvironment` by delegating to the logic Handler in
// `Handler::webview`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	Environment::Requires,
	error::CommonError,
	webview::{WebViewProvider, DTO::*},
};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::webview as WebViewHandler;

#[async_trait]
impl WebViewProvider for MountainEnvironment {
	async fn CreateWebViewPanel(
		&self,
		extension_data:WebViewExtensionDescriptionDTO,
		view_type:String,
		title:String,
		show_options:WebViewShowOptionsDTO,
		panel_options:WebViewPanelOptionsDTO,
		content_options:WebViewContentOptionsDTO,
		serialize_buffers:bool,
	) -> Result<String, CommonError> {
		WebViewHandler::CreateWebViewPanelLogic(
			&self.ApplicationHandle,
			extension_data,
			view_type,
			title,
			show_options,
			panel_options,
			content_options,
			serialize_buffers,
		)
		.await
	}

	async fn DisposeWebView(&self, handle:String) -> Result<(), CommonError> {
		WebViewHandler::DisposeWebViewLogic(&self.ApplicationHandle, handle).await
	}

	async fn RevealWebViewPanel(&self, handle:String, show_options:WebViewShowOptionsDTO) -> Result<(), CommonError> {
		WebViewHandler::RevealWebViewPanelLogic(&self.ApplicationHandle, handle, show_options).await
	}

	async fn SetWebViewTitle(&self, handle:String, title:String) -> Result<(), CommonError> {
		WebViewHandler::SetWebViewTitleLogic(&self.ApplicationHandle, handle, title).await
	}

	async fn SetWebViewIconPath(&self, handle:String, icon_path:Option<Value>) -> Result<(), CommonError> {
		WebViewHandler::SetWebViewIconPathLogic(&self.ApplicationHandle, handle, icon_path).await
	}

	async fn SetWebViewHtml(&self, handle:String, html:String) -> Result<(), CommonError> {
		WebViewHandler::SetWebViewHtmlLogic(&self.ApplicationHandle, handle, html).await
	}

	async fn PostMessageToWebView(&self, handle:String, message:Value) -> Result<bool, CommonError> {
		WebViewHandler::PostMessageToWebViewLogic(&self.ApplicationHandle, handle, message).await
	}
}

impl Requires<Arc<dyn WebViewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebViewProvider + Send + Sync> { Arc::new(self.clone()) }
}
