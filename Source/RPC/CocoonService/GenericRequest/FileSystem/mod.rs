//! Generic-request file-system handlers.
//! `OkResponse`/`ErrResponse` are shared helpers available to sibling modules.

use serde_json::Value;
use tonic::Response;
use ::Vine::Generated::{GenericResponse, RpcError};

pub(super) fn OkResponse(RequestId:u64, Value:&impl serde::Serialize) -> Response<GenericResponse> {
	let Bytes = serde_json::to_vec(Value).unwrap_or_default();

	Response::new(GenericResponse { request_identifier:RequestId, result:Bytes, error:None })
}

pub(super) fn ErrResponse(RequestId:u64, Code:i32, Message:String) -> Response<GenericResponse> {
	Response::new(GenericResponse {
		request_identifier:RequestId,
		result:Vec::new(),
		error:Some(RpcError { code:Code, message:Message, data:Vec::new() }),
	})
}

pub mod CreateDir;

pub mod Delete;

pub mod ReadFile;

pub mod ReadFileUri;

pub mod Readdir;

pub mod ReaddirUri;

pub mod Rename;

pub mod Stat;

pub mod StatUri;

pub mod WriteFile;

pub mod WriteFileUri;
