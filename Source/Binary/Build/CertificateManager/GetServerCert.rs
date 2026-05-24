//! `CertificateManager::GetServerCert`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(This:&Struct, hostname:&str) -> Result<Arc<ServerConfig>> {
		// Check cache first
		{
			let certs = This.server_certs.read();

			if let Some(cert_data) = certs.get(hostname) {
				// Check if certificate is still valid
				if !This.ShouldRenew(&cert_data.cert_pem) {
					dev_log!("security", "using cached server certificate for {}", hostname);

					return Ok(cert_data.server_config.clone());
				}

				// Certificate needs renewal, drop lock and continue
				drop(certs);
			}
		}

		// Generate or renew certificate
		dev_log!("security", "generating server certificate for {}", hostname);

		let cert_data = This.generate_server_cert(hostname)?;

		// Cache the certificate
		{
			let mut certs = This.server_certs.write();

			certs.insert(hostname.to_string(), cert_data.clone());
		}

		Ok(cert_data.server_config)
	}
