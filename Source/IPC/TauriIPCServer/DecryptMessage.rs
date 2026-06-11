//! Verifies the HMAC and decrypts an AES-256-GCM encrypted IPC
//! Message. Body of `SecureMessageChannel::decrypt_message`.

use ring::{
	aead::{self, AES_256_GCM},
	hmac,
};

use super::{EncryptedMessage, SecureMessageChannel, TauriIPCMessage};

pub(crate) fn Fn(Channel:&SecureMessageChannel, encrypted:&EncryptedMessage) -> Result<TauriIPCMessage, String> {
	// Verify HMAC
	let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &Channel.hmac_key);

	hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag)
		.map_err(|_| "HMAC verification failed".to_string())?;

	// Decrypt Message
	let mut in_out = encrypted.ciphertext.clone();

	let nonce_slice:&[u8] = &encrypted.nonce;

	let nonce_array:[u8; 12] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

	let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

	Channel
		.encryption_key
		.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
		.map_err(|e| format!("Decryption failed: {}", e))?;

	// Remove authentication tag
	let plaintext_len = in_out.len() - AES_256_GCM.tag_len();

	in_out.truncate(plaintext_len);

	// Deserialize Message
	serde_json::from_slice(&in_out).map_err(|e| format!("Failed to deserialize Message: {}", e))
}
