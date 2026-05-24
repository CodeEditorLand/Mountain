use std::sync::Arc;

use CommonLibrary::{Debug::DebugService::DebugService, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;
use url::Url;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{I64AtOr, StrAt, StringAt, StringAtOr},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Debug.Start" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn DebugService> = RunTime.Environment.Require();
				let FolderUriStr = StrAt(&Parameters, 0);
				let FolderUri = if FolderUriStr.is_empty() { None } else { Url::parse(FolderUriStr).ok() };
				let Configuration = Parameters.get(1).cloned().unwrap_or_else(|| json!({ "type": "node" }));
				provider
					.StartDebugging(FolderUri, configuration)
					.await
					.map(|SessionId| json!(SessionId))
					.map_err(|E| e.to_string())
			})
		},

		"Debug.RegisterConfigurationProvider" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn DebugService> = RunTime.Environment.Require();
				let DebugType = StringAtOr(&Parameters, 0, "node");
				let ProviderHandle = I64AtOr(&Parameters, 1, 1) as u32;
				let SidecarId = StringAtOr(&Parameters, 2, "cocoon-main");
				provider
					.RegisterDebugConfigurationProvider(DebugType, ProviderHandle, SidecarId)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"Debug.Stop" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn DebugService> = RunTime.Environment.Require();
				let SessionId = StringAt(&Parameters, 0);
				provider
					.StopDebugging(SessionId)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
