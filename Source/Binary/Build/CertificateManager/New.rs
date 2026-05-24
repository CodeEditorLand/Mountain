//! `CertificateManager::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(app_id:&str) -> Result<Self> {
		Ok(Self {
			app_id:app_id.to_string(),
			ca_cert:None,
			ca_key:None,
			server_certs:Arc::new(RwLock::new(HashMap::new())),
		})
	}
