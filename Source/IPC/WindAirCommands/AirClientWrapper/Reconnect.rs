//! `AirClientWrapper::Reconnect`

use super::Struct;
use crate::{Air::Struct as AirClientModule, dev_log};

pub fn Fn(This:&mut Struct, address:String) -> Result<(), String> {
		dev_log!("grpc", "[WindAirCommands] Reconnecting to Air daemon at: {}", address);

		let client = AirClientModule::AirClient::new(&address)
			.await
			.map_err(|E| format!("Failed to reconnect to Air daemon: {:?}", e))?;

		This.Client = client;

		dev_log!("grpc", "[WindAirCommands] Successfully reconnected to Air daemon");

		Ok(())
	}
