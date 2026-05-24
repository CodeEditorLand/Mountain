//! Message compression and secure channel encryption for IPC operations.
//!
//! Gzip compression optimizes message transfer; AES-256-GCM encrypts for
//! confidentiality.

pub mod MessageCompressor;

pub mod SecureChannel;

// Note: Consumers should use Encryption::MessageCompressor::Struct::Struct
// This avoids naming conflicts between module name and type name
