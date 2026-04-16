#![allow(non_snake_case)]
//! Secret Storage domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: get_secret, store_secret, delete_secret.

use CommonLibrary::Secret::SecretProvider::SecretProvider;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::dev_log;
use crate::Vine::Generated::{
	DeleteSecretRequest, Empty, GetSecretRequest, GetSecretResponse,
	StoreSecretRequest,
};

pub async fn GetSecret(
	Service:&CocoonServiceImpl,
	req:GetSecretRequest,
) -> Result<Response<GetSecretResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] get_secret: key={}", req.key);

	// The gRPC proto only carries `key`; we use the app name as the
	// extension identifier (keyring service scoping).
	match Service.environment.GetSecret(String::new(), req.key.clone()).await {
		Ok(Some(Value)) => Ok(Response::new(GetSecretResponse { value:Value })),
		Ok(None) => Ok(Response::new(GetSecretResponse { value:String::new() })),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] get_secret failed key={}: {}", req.key, Error);
			Err(Status::internal(format!("get_secret: {}", Error)))
		},
	}
}

pub async fn StoreSecret(
	Service:&CocoonServiceImpl,
	req:StoreSecretRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] store_secret: key={}", req.key);

	match Service.environment.StoreSecret(String::new(), req.key.clone(), req.value).await {
		Ok(()) => Ok(Response::new(Empty {})),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] store_secret failed key={}: {}", req.key, Error);
			Err(Status::internal(format!("store_secret: {}", Error)))
		},
	}
}

pub async fn DeleteSecret(
	Service:&CocoonServiceImpl,
	req:DeleteSecretRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] delete_secret: key={}", req.key);

	match Service.environment.DeleteSecret(String::new(), req.key.clone()).await {
		Ok(()) => Ok(Response::new(Empty {})),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] delete_secret failed key={}: {}", req.key, Error);
			Err(Status::internal(format!("delete_secret: {}", Error)))
		},
	}
}
