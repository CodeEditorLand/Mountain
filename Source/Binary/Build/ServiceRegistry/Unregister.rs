//! `ServiceRegistry::Unregister`

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
		dev_log!("lifecycle", "[ServiceRegistry] Unregistering service: {}", name);

		if let Ok(mut services) = This.services.write() {
			services.remove(name)
		} else {
			dev_log!(
				"lifecycle",
				"error: [ServiceRegistry] Failed to acquire write lock for unregistration"
			);

			None
		}
	}
