//! Every method returns the same "feature not implemented" error.
//!
//! ## Planned
//!
//! Replace with the real `AirLibrary` client when the feature lands.
pub mod New;
pub mod CheckForUpdates;
pub mod DownloadFile;
pub mod ApplyUpdate;
pub mod AuthenticateUser;
pub mod IndexFiles;
pub mod SearchFiles;
pub mod GetStatus;
pub mod GetMetrics;

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
