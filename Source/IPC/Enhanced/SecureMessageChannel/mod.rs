//! # Secure Message Channel
//!
//! AES-256-GCM + HMAC-SHA256 encrypted IPC channel with
//! automatic key rotation, replay protection, and a generic
//! `SecureMessage::Struct<T>` envelope for adding routing
//! headers. The `Channel::Struct` aggregator + giant impl
//! lives in `Channel.rs` (tightly-coupled cluster); the
//! per-key state, the encrypted-message DTO, the stats DTO,
//! and the secure-message wrapper each live in their own
//! sibling.

pub mod Channel;

pub mod EncryptedMessage;

pub mod EncryptionKey;

pub mod SecureMessage;

pub mod SecurityConfig;

pub mod SecurityStats;
