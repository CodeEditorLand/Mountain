
//! Connect-on-each-call helper. TODO: replace with a state-
//! managed singleton once Air-side connection pooling lands.

use crate::Air::AirClient as AirClientModule;

pub(super) async fn Fn(address:String) -> Result<AirClientModule::AirClient, String> {
	AirClientModule::AirClient::new(&address)
		.await
		.map_err(|e| format!("Failed to create Air client: {:?}", e))
}
