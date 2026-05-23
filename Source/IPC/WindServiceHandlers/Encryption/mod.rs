
//! Encryption handlers for VS Code's `IEncryptionService` channel.
//! `encryption:encrypt` and `encryption:decrypt` are called by the
//! workbench to store and retrieve secrets (extension secrets, auth tokens,
//! GitHub Copilot key, etc.). We use AES-256-GCM via the `ring` crate;
//! the 256-bit key is derived once per process from the machine's hardware
//! UUID (macOS `IOPlatformExpertDevice` / Linux `/etc/machine-id` /
//! Windows `MachineGuid`) so ciphertext is stable across restarts but
//! unreadable on a different machine without the original key.

pub mod Decrypt;

pub mod Encrypt;

pub mod Key;
