//! Connect-on-each-call helper. TODO: replace with a state-
//! managed singleton once Air-side connection pooling lands.

use ::AirLibrary::Client::AirClient::AirClient;

pub(super) async fn Fn(address:String) -> Result<AirClient, String> {

	AirClient::new(&address)
		.await
		.map_err(|e| format!("Failed to create Air client: {:?}", e))
}
