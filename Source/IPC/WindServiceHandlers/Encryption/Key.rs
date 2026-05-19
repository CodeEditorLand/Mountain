#![allow(non_snake_case)]

//! Machine-stable 256-bit key derivation for AES-256-GCM.
//!
//! The key is derived once per process from the host's hardware UUID using
//! SHA-256: `key = SHA-256("Land-Encryption-v1" ++ machine_id)`.
//!
//! Rationale: using the machine ID means ciphertext produced by
//! `encryption:encrypt` survives process restarts (same key each time) but
//! cannot be decrypted on a different machine, matching VS Code's
//! `dpapi`/`safeStorage` semantics. No HSM or external key storage required.

use std::sync::OnceLock;

use ring::digest::{SHA256, digest};

static DERIVED_KEY:[OnceLock<[u8; 32]>; 1] = [OnceLock::new()];

/// Returns the process-wide 256-bit encryption key.
pub fn DeriveKey() -> [u8; 32] { *DERIVED_KEY[0].get_or_init(ComputeKey) }

fn ComputeKey() -> [u8; 32] {
	let MachineId = ReadMachineId();

	let Input = format!("Land-Encryption-v1{}", MachineId);

	let Hash = digest(&SHA256, Input.as_bytes());

	let mut Key = [0u8; 32];

	Key.copy_from_slice(Hash.as_ref());

	Key
}

fn ReadMachineId() -> String {
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
							return Rest[End + 1..].to_string();
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
				return Trimmed;
			}
		}
		if let Ok(Id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
			let Trimmed = Id.trim().to_string();
			if !Trimmed.is_empty() {
				return Trimmed;
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
					return Id.to_string();
				}
			}
		}
	}

	// Fallback: use the executable path hash so at least different installs
	// produce different keys.
	std::env::current_exe()
		.map(|P| format!("{:?}", P))
		.unwrap_or_else(|_| "fallback-land-key-seed".to_string())
}
