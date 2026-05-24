//! `AirClientType::IndexFiles`

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

pub fn Fn(This:&Struct, _Request:IndexRequest::Struct) -> Result<IndexFilesResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}
