//! `CertificateManager::ShouldRenew`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(This:&Struct, cert_pem:&[u8]) -> bool {
		if let Ok(result) = This.check_cert_validity(cert_pem) {
			result.ShouldRenew
		} else {
			// If we can't parse validity, err on the side of renewal
			dev_log!("security", "warn: could not parse certificate validity, forcing renewal");

			true
		}
	}
