//! `CertificateManager::RenewCertificate`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(This:&mut Struct, hostname:&str) -> Result<()> {
		dev_log!("security", "forcing renewal of certificate for {}", hostname);

		// Remove from cache
		let mut certs = This.server_certs.write();

		certs.remove(hostname);

		drop(certs);

		// Generate new certificate
		let cert_data = This.generate_server_cert(hostname)?;

		// Cache the new certificate
		let mut certs = This.server_certs.write();

		certs.insert(hostname.to_string(), cert_data);

		dev_log!("security", "certificate renewed for {}", hostname);

		Ok(())
	}
