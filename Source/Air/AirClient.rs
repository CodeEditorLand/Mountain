//! Mountain-side compat surface for the Air gRPC client. The canonical
//! struct, per-method impls, `IntoRequestExt`, and `Debug` impl live in
//! `Element/Air/Source/Client/AirClient/`. This file re-exposes only
//! what Mountain still references by path:
//!
//! - [`AirClient`] type alias to the canonical type.
//! - [`DEFAULT_AIR_SERVER_ADDRESS`] - gRPC port string (`"[::1]:50053"`).
//! - DTO submodules - each is its own one-line `pub type` alias.
//! - [`MapAirError`] - translation from [`::AirLibrary::AirError`] to
//!   [`CommonError`], used at the Mountain/Air boundary so call sites
//!   keep their `Result<_, CommonError>` signatures and `?` propagation.

pub mod AirMetrics;

pub mod AirStatus;

pub mod DownloadStream;

pub mod DownloadStreamChunk;

pub mod ExtendedFileInfo;

pub mod FileInfo;

pub mod FileResult;

pub mod IndexInfo;

pub mod ResourceUsage;

pub mod UpdateInfo;

use ::AirLibrary::AirError;
use CommonLibrary::Error::CommonError::CommonError;

/// Air gRPC client type. The canonical definition lives in
/// `::AirLibrary::Client::AirClient::AirClient`; every method body, the
/// `new` constructor, the `Debug` impl, and `IntoRequestExt` are owned
/// by the Air crate.
pub type AirClient = ::AirLibrary::Client::AirClient::AirClient;

/// Default gRPC server address for the Air daemon.
///
/// Port allocation:
///
/// - `50051` - Mountain Vine server
/// - `50052` - Cocoon Vine server
/// - `50053` - Air Vine server (this constant)
pub const DEFAULT_AIR_SERVER_ADDRESS:&str = "[::1]:50053";

/// Translate an [`::AirLibrary::AirError`] into a Mountain-side
/// [`CommonError`]. Mountain's existing call sites return
/// `Result<_, CommonError>`; the Air client returns
/// `Result<_, AirError>`. Apply this with `.map_err(MapAirError)` at
/// the boundary.
pub fn MapAirError(Error:AirError) -> CommonError {
	match Error {
		AirError::Authentication(Reason) => CommonError::AccessDenied { Reason },

		AirError::Validation(Reason) => {
			CommonError::InvalidArgument { ArgumentName:"AirRequest".to_string(), Reason }
		},

		AirError::Serialization(Description) => CommonError::SerializationError { Description },

		AirError::ServiceUnavailable(Description) => {
			CommonError::ExternalServiceError { ServiceName:"Air".to_string(), Description }
		},

		AirError::Network(Description)
		| AirError::gRPC(Description)
		| AirError::Configuration(Description)
		| AirError::FileSystem(Description)
		| AirError::Internal(Description)
		| AirError::ResourceLimit(Description)
		| AirError::Timeout(Description)
		| AirError::Plugin(Description)
		| AirError::HotReload(Description)
		| AirError::Connection(Description)
		| AirError::RateLimit(Description)
		| AirError::CircuitBreaker(Description) => CommonError::IPCError { Description },
	}
}
