//! `ServiceRegistry::GetTlsConfig`

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

pub fn Fn(This:&Struct, name:&str) -> Option<std::sync::Arc<rustls::ServerConfig>> {
		let service = This.Lookup(name)?;

		if !service.use_tls {
			return None;
		}

		let cert_manager = This.cert_manager.as_ref()?;

		let manager = cert_manager
			.lock()
			.map_err(|E| {
				dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire lock: {}", e);
			})
			.ok()?;

		manager.BuildServerConfig(name).await.ok()
	}
