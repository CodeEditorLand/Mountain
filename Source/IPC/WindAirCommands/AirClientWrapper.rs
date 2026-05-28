//! gRPC client wrapper - adds reconnect support and PascalCase logging.

use crate::dev_log;

use ::AirLibrary::Client::AirClient::AirClient;

#[derive(Debug, Clone)]
pub struct Struct {
	pub(super) client:AirClient,
}

impl Struct {
	pub async fn new(address:String) -> Result<Self, String> {
		dev_log!("grpc", "[WindAirCommands] Connecting to Air daemon at: {}", address);

		let client = AirClient::new(&address)
			.await
			.map_err(|e| format!("Failed to connect to Air daemon: {:?}", e))?;

		dev_log!("grpc", "[WindAirCommands] Successfully connected to Air daemon");

		Ok(Self { client })
	}

	pub async fn reconnect(&mut self, address:String) -> Result<(), String> {
		dev_log!("grpc", "[WindAirCommands] Reconnecting to Air daemon at: {}", address);

		let client = AirClient::new(&address)
			.await
			.map_err(|e| format!("Failed to reconnect to Air daemon: {:?}", e))?;

		self.client = client;

		dev_log!("grpc", "[WindAirCommands] Successfully reconnected to Air daemon");

		Ok(())
	}
}
