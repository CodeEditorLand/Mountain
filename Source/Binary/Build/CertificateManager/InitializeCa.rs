//! `CertificateManager::InitializeCa`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(This:&mut Struct) -> Result<()> {
		if let Some((cert, key)) = This.load_ca_from_keyring()? {
			dev_log!("security", "loading CA certificate from keyring");

			This.ca_cert = Some(cert.clone());

			This.ca_key = Some(key.clone());

			dev_log!("security", "CA certificate loaded successfully");
		} else {
			dev_log!("security", "CA certificate not found in keyring, generating new CA");

			let (cert, key) = This.generate_ca_cert()?;

			// Store in keyring
			This.save_ca_to_keyring(&cert, &key)?;

			This.ca_cert = Some(cert.clone());

			This.ca_key = Some(key);

			dev_log!("security", "new CA certificate generated and stored");
		}

		Ok(())
	}
