//! Return an authentication session for the requested provider. Cocoon
//! auth providers register themselves via `RegisterAuthenticationProvider`
//! and live in `ApplicationState`. If a provider with the requested id is
//! registered, Mountain forwards the call to Cocoon's
//! `ExtHostAuthentication$getSession` and maps the response back to proto
//! fields. If no provider is registered the default (empty) response is
//! returned, which VS Code treats as "no session available".

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
use serde_json::json;
use tonic::{Response, Status};
use ::Vine::Generated::{GetAuthenticationSessionRequest, GetAuthenticationSessionResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, Vine::Client::SendRequest, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:GetAuthenticationSessionRequest,
) -> Result<Response<GetAuthenticationSessionResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] get_authentication_session: provider={}",
		Request.provider_id
	);

	// Look up a registered authentication provider whose Selector encodes the
	// requested provider_id (stored as `[{"provider": "<id>"}]` by
	// RegisterAuthenticationProvider). Iterate all registered providers rather
	// than reconstructing the hash so the lookup is hash-algorithm-independent.
	let ProviderFound = Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.GetProviders()
		.into_values()
		.any(|dto| {
			if dto.ProviderType != ProviderType::Authentication {
				return false;
			}

			// Selector shape: [{"provider": "<id>"}]
			dto.Selector
				.as_array()
				.and_then(|arr| arr.first())
				.and_then(|entry| entry.get("provider"))
				.and_then(|v| v.as_str())
				.map_or(false, |id| id == Request.provider_id)
		});

	if !ProviderFound {
		dev_log!(
			"cocoon",
			"[CocoonService] get_authentication_session: no provider registered for '{}'",
			Request.provider_id
		);

		return Ok(Response::new(GetAuthenticationSessionResponse::default()));
	}

	// Forward to Cocoon's ExtHostAuthentication$getSession. Payload mirrors the
	// VS Code extension-host wire format: [providerId, scopes, options].
	let Payload = json!([
		Request.provider_id,
		Request.scopes,
		{
			"createIfNone": Request.create_if_none,
			"clearSessionPreference": Request.clear_session_preference,
			"forceNewSession": false,
		}
	]);

	match SendRequest::Fn("cocoon-main", "ExtHostAuthentication$getSession".to_string(), Payload, 10000).await {
		Ok(Session) if !Session.is_null() => {
			let id = Session.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

			let access_token = Session.get("accessToken").and_then(|v| v.as_str()).unwrap_or("").to_string();

			let account_label = Session
				.get("account")
				.and_then(|a| a.get("label"))
				.and_then(|v| v.as_str())
				.unwrap_or("")
				.to_string();

			let account_id = Session
				.get("account")
				.and_then(|a| a.get("id"))
				.and_then(|v| v.as_str())
				.unwrap_or("")
				.to_string();

			let scopes = Session
				.get("scopes")
				.and_then(|v| v.as_array())
				.map(|arr| {
					arr.iter()
						.filter_map(|s| s.as_str())
						.map(|s| s.to_string())
						.collect::<Vec<String>>()
				})
				.unwrap_or_default();

			dev_log!(
				"cocoon",
				"[CocoonService] get_authentication_session: session id='{}' for provider='{}'",
				id,
				Request.provider_id
			);

			Ok(Response::new(GetAuthenticationSessionResponse {
				id,
				access_token,
				account_label,
				account_id,
				scopes,
			}))
		},

		Ok(_) => {
			// Cocoon returned null - provider active but no session available.
			dev_log!(
				"cocoon",
				"[CocoonService] get_authentication_session: provider='{}' returned no session",
				Request.provider_id
			);

			Ok(Response::new(GetAuthenticationSessionResponse::default()))
		},

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] get_authentication_session: Cocoon error for provider='{}': {:?}",
				Request.provider_id,
				Error
			);

			Ok(Response::new(GetAuthenticationSessionResponse::default()))
		},
	}
}
