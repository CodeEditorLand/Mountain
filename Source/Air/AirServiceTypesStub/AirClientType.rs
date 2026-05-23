//! Stub `AirClient` type used while the `AirIntegration` feature is off.
//! Every method returns the same "feature not implemented" error.
//!
//! ## Planned
//!
//! Replace with the real `AirLibrary` client when the feature lands.

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

#[derive(Debug, Clone)]
pub struct Struct;

impl Struct {
	pub async fn new(_Address:&str) -> Result<Self, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn CheckForUpdates(
		&self,

		_Request:UpdateCheckRequest::Struct,
	) -> Result<UpdateCheckResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn DownloadFile(&self, _Request:DownloadRequest::Struct) -> Result<DownloadFileResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn ApplyUpdate(
		&self,

		_Request:ApplyUpdateRequest::Struct,
	) -> Result<ApplyUpdateResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn AuthenticateUser(
		&self,

		_Request:AuthenticationRequest::Struct,
	) -> Result<AuthenticationResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn IndexFiles(&self, _Request:IndexRequest::Struct) -> Result<IndexFilesResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn SearchFiles(&self, _Request:SearchRequest::Struct) -> Result<SearchFilesResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn GetStatus(&self, _Request:StatusRequest::Struct) -> Result<StatusResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}

	pub async fn GetMetrics(&self, _Request:MetricsRequest::Struct) -> Result<MetricsResponse::Struct, String> {
		Err("AirIntegration feature is not implemented yet".to_string())
	}
}
