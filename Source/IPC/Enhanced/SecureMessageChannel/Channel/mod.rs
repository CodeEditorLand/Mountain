pub mod New;
pub mod Start;
pub mod Stop;
pub mod EncryptMessage;
pub mod DecryptMessage;
pub mod RotateKeys;
pub mod GetStats;
pub mod ValidateMessageIntegrity;
pub mod DefaultChannel;
pub mod HighSecurityChannel;
pub mod GenerateSecureKey;
pub mod CalculateEncryptionOverhead;
pub mod EstimateEncryptedSize;
pub mod CreateSecureMessage;

use std::{
	collections::HashMap,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, SystemTime},
};

use bincode::serde::{decode_from_slice, encode_to_vec};
use ring::{
	aead::{self, AES_256_GCM, NONCE_LEN},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
	IPC::Enhanced::SecureMessageChannel::{
		EncryptedMessage::Struct as EncryptedMessage,
		EncryptionKey::Struct as EncryptionKey,
		SecureMessage::Struct as SecureMessage,
		SecurityConfig::Struct as SecurityConfig,
		SecurityStats::Struct as SecurityStats,
	},
	dev_log,
};

pub struct Struct {
	pub config:SecurityConfig,

	pub current_key:Arc<RwLock<EncryptionKey>>,

	pub previous_keys:Arc<RwLock<HashMap<String, EncryptionKey>>>,

	pub hmac_key:Arc<RwLock<Vec<u8>>>,

	pub rng:SystemRandom,

	pub key_rotation_task:Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}
