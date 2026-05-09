#![allow(non_snake_case)]

//! Forward a code-actions request to the registered provider. Currently
//! returns an empty list pending the action-DTO mapping.

use serde_json::json;

use tonic::{Response, Status};

use url::Url;

use CommonLibrary::LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ProvideCodeActionsRequest, ProvideCodeActionsResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideCodeActionsRequest,
) -> Result<Response<ProvideCodeActionsResponse>, Status> {

	dev_log!(
		"cocoon",

		"[CocoonService] Providing code actions for provider {}",

		Request.provider_handle
	);

	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let R = Request.range.as_ref();

	let RangeDTO = json!({
		"StartLineNumber": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
		"StartColumn": R.and_then(|R| R.start.as_ref()).map(|P| P.character).unwrap_or(0),
		"EndLineNumber": R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
		"EndColumn": R.and_then(|R| R.end.as_ref()).map(|P| P.character).unwrap_or(0),
	});

	let ContextDTO = json!({ "diagnostics": [], "only": null });

	match Service.environment.ProvideCodeActions(DocumentURI, RangeDTO, ContextDTO).await {

		Ok(_) => Ok(Response::new(ProvideCodeActionsResponse { actions:Vec::new() })),

		Err(Error) => Err(Status::internal(format!("Code actions failed: {}", Error))),
	}
}
