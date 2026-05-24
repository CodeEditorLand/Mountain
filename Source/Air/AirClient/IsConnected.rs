//! `AirClient::IsConnected`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};
use crate::dev_log;

pub fn Fn(This:&Struct) -> bool {
		#[cfg(feature = "AirIntegration")]
		{
			This.Client.is_some()
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			false
		}
	}
