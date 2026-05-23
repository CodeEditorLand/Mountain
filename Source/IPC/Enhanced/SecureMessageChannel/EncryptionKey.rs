
//! Wrapper around `ring::aead::LessSafeKey` plus metadata -
//! creation timestamp, random key id, and a usage counter the
//! channel bumps on each encrypt. Private constructors are
//! exposed via `pub(super)` so the channel can manage rotation
//! while keeping callers out of the raw key material.

use std::time::{Duration, SystemTime};

use ring::{
	aead::{AES_256_GCM, LessSafeKey, UnboundKey},
	rand::{SecureRandom, SystemRandom},
};

#[derive(Debug, Clone)]
pub struct Struct {
	pub(super) key:LessSafeKey,

	pub(super) created_at:SystemTime,

	pub(super) key_id:String,

	pub(super) usage_count:usize,
}

impl Struct {
	pub(super) fn new(key_bytes:&[u8]) -> Result<Self, String> {
		let unbound_key =
			UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|e| format!("Failed to create unbound key: {}", e))?;

		Ok(Self {
			key:LessSafeKey::new(unbound_key),
			created_at:SystemTime::now(),
			key_id:Self::generate_key_id(),
			usage_count:0,
		})
	}

	fn generate_key_id() -> String {
		let rng = SystemRandom::new();

		let mut id_bytes = [0u8; 8];

		rng.fill(&mut id_bytes).unwrap();

		hex::encode(id_bytes)
	}

	pub(super) fn is_expired(&self, rotation_interval:Duration) -> bool {
		self.created_at.elapsed().unwrap_or_default() > rotation_interval
	}

	pub(super) fn increment_usage(&mut self) { self.usage_count += 1; }
}
