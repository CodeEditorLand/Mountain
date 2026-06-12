use ::AirLibrary::AirError;
use CommonLibrary::Error::CommonError::CommonError;

/// Alias for the AirLibrary client type.
pub type AirClient = ::AirLibrary::Client::AirClient::AirClient;

/// Maps an AirLibrary error to a CommonError.
pub fn MapAirError(Error:AirError) -> CommonError {
	match Error {
		AirError::Authentication(Reason) => CommonError::AccessDenied { Reason },

		AirError::Validation(Reason) => CommonError::InvalidArgument { ArgumentName:"AirRequest".to_string(), Reason },

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
