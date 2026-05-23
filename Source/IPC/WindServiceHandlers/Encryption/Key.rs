
//! Machine-stable 256-bit key derivation for AES-256-GCM.
//!
//! The key is derived once per process from the host's hardware UUID using
//! SHA-256: `key = SHA-256("Land-Encryption-v1" ++ machine_id)`.
//!
//! If the machine ID cannot be obtained on this system, `DeriveKey` returns
//! `Err` so callers can surface a meaningful error rather than silently
//! falling back to a constant key that is identical on every affected machine.

use std::sync::OnceLock;

use ring::digest::{SHA256, digest};

static DERIVED_KEY:OnceLock<Option<[u8; 32]>> = OnceLock::new();

/// Returns the process-wide 256-bit encryption key.
///
/// Returns `Err` when no machine-bound seed is available on this system so
/// that callers can propagate the failure instead of encrypting with a
/// predictable constant.
pub fn Fn() -> Result<[u8; 32], String> {
	DERIVED_KEY
		.get_or_init(ComputeKey)
		.ok_or_else(|| "encryption unavailable: machine ID lookup failed".to_string())
}

fn ComputeKey() -> Option<[u8; 32]> {
	let MachineId = ReadMachineId()?;

	let Input = format!("Land-Encryption-v1{}", MachineId);

	let Hash = digest(&SHA256, Input.as_bytes());

	let mut Key = [0u8; 32];

	Key.copy_from_slice(Hash.as_ref());

	Some(Key)
}

fn ReadMachineId() -> Option<String> {
	#[cfg(target_os = "macos")]
	{
		if let Ok(Out) = std::process::Command::new("ioreg")
			.args(["-rd1", "-c", "IOPlatformExpertDevice"])
			.output()
		{
			let S = String::from_utf8_lossy(&Out.stdout);

			for Line in S.lines() {
				if Line.contains("IOPlatformUUID") {
					if let Some(Start) = Line.rfind('"') {
						let Rest = &Line[..Start];

						if let Some(End) = Rest.rfind('"') {
							return Some(Rest[End + 1..].to_string());
						}
					}
				}
			}
		}
	}

	#[cfg(target_os = "linux")]
	{
		if let Ok(Id) = std::fs::read_to_string("/etc/machine-id") {
			let Trimmed = Id.trim().to_string();

			if !Trimmed.is_empty() {
				return Some(Trimmed);
			}
		}

		if let Ok(Id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
			let Trimmed = Id.trim().to_string();

			if !Trimmed.is_empty() {
				return Some(Trimmed);
			}
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		if let Ok(Out) = Command::new("reg")
			.args(["query", "HKLM\\SOFTWARE\\Microsoft\\Cryptography", "/v", "MachineGuid"])
			.output()
		{
			let S = String::from_utf8_lossy(&Out.stdout);

			if let Some(Line) = S.lines().find(|L| L.contains("MachineGuid")) {
				if let Some(Id) = Line.split_whitespace().last() {
					return Some(Id.to_string());
				}
			}
		}
	}

	None
}
