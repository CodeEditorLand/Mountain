use CommonLibrary::IPC::DTO::ProxyTarget::ProxyTarget;
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::{
			Params::StringAt,
			Proxy::crate::Track::Effect::CreateEffectForRequest::Utilities::Proxy::Fn,
		},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Authentication.GetSession" => {
			crate::effect!(RunTime, {
				let ProviderId = StringAt(&Parameters, 0);
				let Scopes = Parameters.get(1).cloned().unwrap_or(json!([]));
				let Options = Parameters.get(2).cloned().unwrap_or(json!({}));
				crate::Track::Effect::CreateEffectForRequest::Utilities::Proxy::Fn(
					&RunTime,
					ProxyTarget::ExtHostAuthentication,
					"getSession",
					json!([ProviderId, scopes, options]),
					5000,
				)
				.await
				.or_else(|Error| {
					dev_log!(
						"ipc",
						"warn: [Authentication.GetSession] extension did not answer ({:?}); returning null",
						error
					);
					Ok(json!(null))
				})
			})
		},

		"Authentication.GetAccounts" => {
			crate::effect!(RunTime, {
				let ProviderId = StringAt(&Parameters, 0);
				crate::Track::Effect::CreateEffectForRequest::Utilities::Proxy::Fn(
					&RunTime,
					ProxyTarget::ExtHostAuthentication,
					"getAccounts",
					json!([ProviderId]),
					5000,
				)
				.await
				.or_else(|Error| {
					dev_log!(
						"ipc",
						"warn: [Authentication.GetAccounts] extension did not answer ({:?}); returning []",
						error
					);
					Ok(json!([]))
				})
			})
		},

		_ => None,
	}
}
