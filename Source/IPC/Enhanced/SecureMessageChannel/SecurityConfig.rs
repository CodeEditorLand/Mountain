//! Tunables for the secure-message channel - encryption /
//! HMAC algorithm, key-rotation cadence, nonce / tag sizes,
//! and the maximum allowed message size (DOS guard).

use ring::aead::{AES_256_GCM, NONCE_LEN};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub encryption_algorithm:String,

	pub key_rotation_interval_hours:u64,

	pub hmac_algorithm:String,

	pub nonce_size_bytes:usize,

	pub auth_tag_size_bytes:usize,

	pub max_message_size_bytes:usize,
}

impl Default for Struct {
	fn default() -> Self {
		Self {
			encryption_algorithm:"AES-256-GCM".to_string(),

			key_rotation_interval_hours:24,

			hmac_algorithm:"HMAC-SHA256".to_string(),

			nonce_size_bytes:NONCE_LEN,

			auth_tag_size_bytes:AES_256_GCM.tag_len(),

			max_message_size_bytes:10 * 1024 * 1024,
		}
	}
}
