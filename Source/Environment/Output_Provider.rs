// ---------------------------------------------------------------------------------------------
// Mountain Environment - Output Channel Provider
// 
// --------------------------------------------------------------------------------------------
// This module implements the `OutputChannelManager` trait for
// `MountainEnvironment`. It manages output channels used for displaying logs
// and textual information from extensions or system processes. Operations are
// delegated to handler functions in `handlers::output`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	environment::Requires,
	errors::CommonError,
	output_effects::OutputChannelManager, // The trait being implemented
};
use async_trait::async_trait;
use log::{info, trace}; // For logging

use crate::{
	environment::MountainEnvironment,
	handlers, // For delegating to output handlers
};

// --- OutputChannelManager Implementation ---
#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	async fn register_channel(&self, name:String, language_id:Option<String>) -> Result<String, CommonError> {
		info!(
			"[Env OutputProv] RegisterChannel: name='{}', language_id='{:?}'",
			name, language_id
		);

		// Delegate to the handler function.
		handlers::output::handle_register_output_channel_effect_logic(self.app_handle.clone(), name, language_id).await
	}

	async fn append(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		trace!(
			"[Env OutputProv] Append: channel_id='{}', value_len={}",
			channel_id,
			value.len()
		);

		// Delegate to the handler function.
		handlers::output::handle_append_to_output_channel_effect_logic(self.app_handle.clone(), channel_id, value).await
	}

	async fn replace(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		info!(
			"[Env OutputProv] Replace: channel_id='{}', new_value_len={}",
			channel_id,
			value.len()
		);
		handlers::output::handle_replace_output_channel_content_effect_logic(self.app_handle.clone(), channel_id, value)
			.await
	}

	async fn clear(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputProv] Clear: channel_id='{}'", channel_id);
		handlers::output::handle_clear_output_channel_effect_logic(self.app_handle.clone(), channel_id).await
	}

	async fn reveal(&self, channel_id:String, preserve_focus:bool) -> Result<(), CommonError> {
		info!(
			"[Env OutputProv] Reveal: channel_id='{}', preserve_focus={}",
			channel_id, preserve_focus
		);
		handlers::output::handle_reveal_output_channel_effect_logic(self.app_handle.clone(), channel_id, preserve_focus)
			.await
	}

	async fn close(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputProv] Close: channel_id='{}'", channel_id);
		handlers::output::handle_close_output_channel_view_effect_logic(self.app_handle.clone(), channel_id).await
	}

	async fn dispose(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputProv] Dispose: channel_id='{}'", channel_id);
		handlers::output::handle_dispose_output_channel_effect_logic(self.app_handle.clone(), channel_id).await
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}
