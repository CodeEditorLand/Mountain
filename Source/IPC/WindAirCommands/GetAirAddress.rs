//! Resolve the Air daemon's gRPC address. Currently hard-coded
//! to `DEFAULT_AIR_SERVER_ADDRESS`; future revisions will read
//! a runtime config slot.

pub(super) fn Fn() -> Result<String, String> {

	Ok(::AirLibrary::Client::AirClient::DEFAULT_AIR_SERVER_ADDRESS.to_string())
}
