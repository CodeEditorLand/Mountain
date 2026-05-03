#![allow(non_snake_case)]

//! gRPC client wrapper around `Air::AirClient::AirClient` -
//! adds reconnect support and PascalCase logging consistent
//! with the WindAirCommands surface.

use crate::{Air::AirClient as AirClientModule, dev_log};

#[derive(Debug, Clone)]
pub struct Struct {
	pub(super) client:AirClientModule::AirClient,
}

impl Struct {
	pub async fn new(address:String) -> Result<Self, String> {
		dev_log!("grpc", "[WindAirCommands] Connecting to Air daemon at: {}", address);

		let client = AirClientModule::AirClient::new(&address)
			.await
			.map_err(|e| format!("Failed to connect to Air daemon: {:?}", e))?;

		dev_log!("grpc", "[WindAirCommands] Successfully connected to Air daemon");
		Ok(Self { client })
	}

	pub async fn reconnect(&mut self, address:String) -> Result<(), String> {
		dev_log!("grpc", "[WindAirCommands] Reconnecting to Air daemon at: {}", address);

		let client = AirClientModule::AirClient::new(&address)
			.await
			.map_err(|e| format!("Failed to reconnect to Air daemon: {:?}", e))?;

		self.client = client;
		dev_log!("grpc", "[WindAirCommands] Successfully reconnected to Air daemon");
		Ok(())
	}
}
