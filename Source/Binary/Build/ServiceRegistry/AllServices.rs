//! `ServiceRegistry::AllServices`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use http::{Request as HttpRequest, Response as HttpResponse, header};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Vec<LocalService> {
		if let Ok(services) = This.services.read() {
			services.values().cloned().collect()
		} else {
			dev_log!(
				"lifecycle",
				"error: [ServiceRegistry] Failed to acquire read lock for all_services"
			);

			Vec::new()
		}
	}
