//! `AirClientWrapper::New`

use super::Struct;
use crate::{Air::Struct as AirClientModule, dev_log};

pub fn Fn(address:String) -> Result<Self, String> {
		dev_log!("grpc", "[WindAirCommands] Connecting to Air daemon at: {}", address);

		let client = AirClientModule::AirClient::new(&address)
			.await
			.map_err(|E| format!("Failed to connect to Air daemon: {:?}", e))?;

		dev_log!("grpc", "[WindAirCommands] Successfully connected to Air daemon");

		Ok(Self { client })
	}
