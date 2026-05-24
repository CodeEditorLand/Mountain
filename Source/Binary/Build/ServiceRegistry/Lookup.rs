//! `ServiceRegistry::Lookup`

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

pub fn Fn(This:&Struct, name:&str) -> Option<LocalService> {
		dev_log!("lifecycle", "[ServiceRegistry] Looking up service: {}", name);

		if let Ok(services) = This.services.read() {
			let service = services.get(name).cloned();

			if service.is_some() {
				dev_log!("lifecycle", "[ServiceRegistry] Service {} found", name);
			} else {
				dev_log!("lifecycle", "[ServiceRegistry] Service {} not found", name);
			}

			service
		} else {
			dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire read lock for lookup");

			None
		}
	}
