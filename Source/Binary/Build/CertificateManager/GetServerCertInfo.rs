//! `CertificateManager::GetServerCertInfo`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(This:&Struct, hostname:&str) -> Option<CertificateInfo> {
		let certs = This.server_certs.read();

		certs.get(hostname).map(|d| d.info.clone())
	}
