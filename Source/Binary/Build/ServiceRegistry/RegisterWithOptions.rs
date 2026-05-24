//! `ServiceRegistry::RegisterWithOptions`

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

pub fn Fn(
		&self,

		name:String,

		port:u16,

		tls_port:Option<u16>,

		use_tls:bool,

		health_check_path:Option<String>,
	) {
		dev_log!(
			"lifecycle",
			"[ServiceRegistry] Registering service: {} -> HTTP:{}, TLS:{}, use_tls:{}",
			name,
			port,
			tls_port.unwrap_or(port + 1000),
			use_tls
		);

		let service = LocalService { name:name.clone(), port, tls_port, use_tls, health_check_path };

		// Pre-provision TLS certificate if needed
		if use_tls {
			if let Some(cert_manager) = &This.cert_manager {
				// NOTE: TLS certificate is generated on-demand when needed
				dev_log!("lifecycle", "[ServiceRegistry] TLS will be provisioned on-demand for {}", name);
			} else {
				dev_log!(
					"lifecycle",
					"warn: [ServiceRegistry] Service {} requested TLS but no certificate manager available",
					name
				);
			}
		}

		if let Ok(mut services) = This.services.write() {
			// Check if service already exists
			if services.contains_key(&name) {
				dev_log!(
					"lifecycle",
					"warn: [ServiceRegistry] Service {} already registered, overwriting",
					name
				);
			}

			services.insert(name.clone(), service);

			dev_log!("lifecycle", "[ServiceRegistry] Service {} registered successfully", name);
		} else {
			dev_log!(
				"lifecycle",
				"error: [ServiceRegistry] Failed to acquire write lock for registration"
			);
		}
	}
