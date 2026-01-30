// File: Mountain/Source/Air/mod.rs
// Role: Air gRPC client module for Mountain
// Responsibilities:
//   - Provide gRPC client connectivity to the Air daemon service
//   - Implement Air service methods for update management, authentication,
//     file indexing, and system monitoring
//   - Handle connection management and error translation to CommonError
//   - Wrap client in Arc for shared access across the application

#![allow(non_snake_case, non_camel_case_types)]

pub mod AirClient;
pub mod AirServiceProvider;

// Re-export convenience types from submodules
pub use AirClient::{
	AuthenticationRequest,
	AuthenticationResponse,
	UpdateCheckRequest,
	UpdateCheckResponse,
	ApplyUpdateRequest,
	ApplyUpdateResponse,
	DownloadRequest,
	DownloadResponse,
	IndexRequest,
	IndexResponse,
	SearchRequest,
	SearchResponse,
	StatusRequest,
	StatusResponse,
	MetricsRequest,
	MetricsResponse,
	DEFAULT_AIR_SERVER_ADDRESS,
};
pub use AirServiceProvider::{CreateAirServiceProvider, CreateAirServiceProviderOrUnavailable};
