
//! Serialised encrypted-message envelope - key id (so
//! decryption can find the right key during rotation), nonce,
//! AES-256-GCM ciphertext, HMAC tag, and a millisecond
//! timestamp used for replay-window enforcement.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub key_id:String,

	pub nonce:Vec<u8>,

	pub ciphertext:Vec<u8>,

	pub hmac_tag:Vec<u8>,

	pub timestamp:u64,
}
