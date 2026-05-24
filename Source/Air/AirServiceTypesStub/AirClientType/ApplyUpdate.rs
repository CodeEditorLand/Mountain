//! `AirClientType::ApplyUpdate`

use super::Struct;
use crate::Air::AirServiceTypesStub::{
	ApplyUpdateRequest,
	ApplyUpdateResponse,
	AuthenticationRequest,
	AuthenticationResponse,
	DownloadFileResponse,
	DownloadRequest,
	IndexFilesResponse,
	IndexRequest,
	MetricsRequest,
	MetricsResponse,
	SearchFilesResponse,
	SearchRequest,
	StatusRequest,
	StatusResponse,
	UpdateCheckRequest,
	UpdateCheckResponse,
};

pub fn Fn(
		&self,

		_Request:ApplyUpdateRequest::Struct,
	) -> Result<ApplyUpdateResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}
