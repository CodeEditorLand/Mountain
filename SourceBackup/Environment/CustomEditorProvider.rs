// @module CustomEditorProvider (Environment)
// @description Implements the `CustomEditorProvider` trait for
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	custom_editor::{CustomEditorProvider, DTO::*},
	Environment::Requires,
	error::CommonError,
};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::custom_editor as CustomEditorHandler;

#[async_trait]
impl CustomEditorProvider for MountainEnvironment {
	async fn RegisterCustomEditor(
		&self,
		view_type:String,
		options:CustomEditorOptionsDTO,
		extension_id:String,
		sidecar_id:String,
	) -> Result<(), CommonError> {
		CustomEditorHandler::RegisterCustomEditorLogic(
			&self.ApplicationHandle,
			view_type,
			options,
			extension_id,
			sidecar_id,
		)
		.await
	}

	async fn UnregisterCustomEditor(&self, view_type:String) -> Result<(), CommonError> {
		CustomEditorHandler::UnregisterCustomEditorLogic(&self.ApplicationHandle, view_type).await
	}

	async fn CreateCustomDocument(&self, resource_uri:Value, view_type:String) -> Result<Value, CommonError> {
		CustomEditorHandler::CreateCustomDocumentLogic(&self.ApplicationHandle, resource_uri, view_type).await
	}
}

impl Requires<Arc<dyn CustomEditorProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CustomEditorProvider + Send + Sync> { Arc::new(self.clone()) }
}
