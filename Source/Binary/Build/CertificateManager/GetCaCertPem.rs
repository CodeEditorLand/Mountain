//! `CertificateManager::GetCaCertPem`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Option<Vec<u8>> { This.ca_cert.clone() }
