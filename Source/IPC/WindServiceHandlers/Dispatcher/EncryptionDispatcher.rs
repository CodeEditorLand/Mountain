//! Encryption command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::Encryption::{Decrypt::Fn as Decrypt, Encrypt::Fn as Encrypt};
=======
use crate::IPC::WindServiceHandlers::Encryption::{Decrypt::Fn as Decrypt, Encrypt::Fn as Encrypt};
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

/// Dispatches encryption commands.
///
/// Handled commands:
/// - `encryption:encrypt`
/// - `encryption:decrypt`
pub async fn dispatch_encryption(arguments:Vec<Value>, command:&str) -> Result<Value, String> {
	match command {
		"encryption:encrypt" => Encrypt(arguments).await,

		"encryption:decrypt" => Decrypt(arguments).await,

		_ => Err(format!("Unknown encryption command: {}", command)),
	}
}
