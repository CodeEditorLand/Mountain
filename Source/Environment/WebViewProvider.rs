//! # WebViewProvider Implementation
//!
//! Implements the `WebViewProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for creating and managing WebView
//! instances.

use Common::{Error::CommonError, WebView::WebViewProvider};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;

use super::MountainEnvironment;

#[async_trait]
impl WebViewProvider for MountainEnvironment {
	async fn CreateWebViewPanel(
		&self,
		_ExtensionData:Value,
		_ViewType:String,
		_Title:String,
		_ShowOptions:Value,
		_PanelOptions:Value,
		_ContentOptions:Value,
	) -> Result<String, CommonError> {
		warn!("[WebViewProvider] CreateWebViewPanel is not implemented.");
		Err(CommonError::NotImplemented { FeatureName:"CreateWebViewPanel".into() })
	}

	async fn DisposeWebView(&self, _Handle:String) -> Result<(), CommonError> {
		warn!("[WebViewProvider] DisposeWebView is not implemented.");
		Ok(())
	}

	async fn RevealWebViewPanel(&self, _Handle:String, _ShowOptions:Value) -> Result<(), CommonError> {
		warn!("[WebViewProvider] RevealWebViewPanel is not implemented.");
		Ok(())
	}

	async fn SetWebViewTitle(&self, _Handle:String, _Title:String) -> Result<(), CommonError> {
		warn!("[WebViewProvider] SetWebViewTitle is not implemented.");
		Ok(())
	}

	async fn SetWebViewIconPath(&self, _Handle:String, _IconPath:Option<Value>) -> Result<(), CommonError> {
		warn!("[WebViewProvider] SetWebViewIconPath is not implemented.");
		Ok(())
	}

	async fn SetWebViewHTML(&self, _Handle:String, _HTML:String) -> Result<(), CommonError> {
		warn!("[WebViewProvider] SetWebViewHTML is not implemented.");
		Ok(())
	}

	async fn PostMessageToWebView(&self, _Handle:String, _Message:Value) -> Result<bool, CommonError> {
		warn!("[WebViewProvider] PostMessageToWebView is not implemented.");
		Ok(true)
	}
}
