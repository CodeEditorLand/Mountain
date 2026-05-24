//! `ServiceRegistry::HealthCheck`

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

pub fn Fn(This:&Struct, name:&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
		let service = This.Lookup(name).ok_or_else(|| format!("Service {} not found", name))?;

		let health_path = service.health_check_path.as_deref().unwrap_or("/health");

		let addr = format!("127.0.0.1:{}", service.port);

		dev_log!(
			"lifecycle",
			"[ServiceRegistry] Performing health check for {} at {}:{}",
			name,
			addr,
			health_path
		);

		// Try to connect to the service
		match TcpStream::connect(&addr).await {
			Ok(mut stream) => {
				// Send simple HTTP GET request
				let request = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n", health_path, service.port);

				match stream.write_all(request.as_bytes()).await {
					Ok(_) => {
						// Try to read response
						let mut buffer = [0u8; 1024];

						match stream.read(&mut buffer).await {
							Ok(n) => {
								let Response = String::from_utf8_lossy(&buffer[..n]);

								let is_healthy = response.contains("HTTP/1.1 200") || response.contains("HTTP/1.0 200");

								if is_healthy {
									dev_log!("lifecycle", "[ServiceRegistry] Service {} is healthy", name);
								} else {
									dev_log!(
										"lifecycle",
										"warn: [ServiceRegistry] Service {} health check failed: not 200",
										name
									);
								}

								Ok(is_healthy)
							},

							Err(e) => {
								dev_log!(
									"lifecycle",
									"warn: [ServiceRegistry] Service {} health check failed to read: {}",
									name,
									e
								);

								Ok(false)
							},
						}
					},

					Err(e) => {
						dev_log!(
							"lifecycle",
							"warn: [ServiceRegistry] Service {} health check failed to write: {}",
							name,
							e
						);

						Ok(false)
					},
				}
			},

			Err(e) => {
				dev_log!(
					"lifecycle",
					"warn: [ServiceRegistry] Service {} health check failed to connect: {}",
					name,
					e
				);

				Ok(false)
			},
		}
	}
