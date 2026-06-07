pub fn Matches(MethodName:&str) -> bool {

	match MethodName {
		"Authentication.GetSession" | "Authentication.GetAccounts" => true,

		_ => false,
	}
}

use CommonLibrary::IPC::DTO::ProxyTarget::ProxyTarget;

use serde_json::{Value, json};

use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::{Params::string_at, Proxy::proxy_cocoon},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {

	match MethodName {
		"Authentication.GetSession" => {
			crate::effect!(run_time, {
				let provider_id = string_at(&Parameters, 0);

				let scopes = Parameters.get(1).cloned().unwrap_or(json!([]));

				let options = Parameters.get(2).cloned().unwrap_or(json!({}));

				proxy_cocoon(
					&run_time,

					ProxyTarget::ExtHostAuthentication,

					"getSession",

					json!([provider_id, scopes, options]),

					5000,
				)
				.await
				.or_else(|error| {
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
			crate::effect!(run_time, {
				let provider_id = string_at(&Parameters, 0);

				proxy_cocoon(
					&run_time,

					ProxyTarget::ExtHostAuthentication,

					"getAccounts",

					json!([provider_id]),

					5000,
				)
				.await
				.or_else(|error| {
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
