//! # Encryption Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides message compression and secure channel encryption for
//! IPC operations. It optimizes message transfer through compression and ensures
//! message confidentiality through AES-256-GCM encryption.
//!
//! ## ARCHITECTURAL ROLE
//! This module is part of the security and performance layer in the IPC architecture,
//! providing compression for efficiency and encryption for confidentiality.
//!
//! ## KEY COMPONENTS
//!
//! - **MessageCompressor**: Gzip compression for efficient message transfer
//! - **SecureMessageChannel**: AES-256-GCM encryption for secure communication
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages for
//! compression/encryption failures.
//!
//! ## LOGGING
//! Debug-level logging for compression ratios, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Compression level 6 provides good balance between speed and ratio
//! - Batch size 10 aggregates small messages for efficiency
//! - AES-256-GCM provides hardware-accelerated encryption on modern CPUs
//!
//! ## TODO
//! - Add compression algorithm selection
//! - Implement compression ratio optimization
//! - Add encryption key rotation
//! - Implement message authentication codes

pub mod MessageCompressor;
pub mod SecureChannel;

// Note: Consumers should use Encryption::MessageCompressor::MessageCompressor
// This avoids naming conflicts between module name and type name
