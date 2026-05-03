#![allow(non_snake_case)]

//! # Message Compressor and Batching
//!
//! Buffers IPC messages into batches, then compresses on
//! flush using Brotli / Gzip / Zlib at the configured level.
//! `Compressor::Struct` is the engine; the DTOs are split into
//! their own siblings.

pub mod BatchConfig;
pub mod BatchStats;
pub mod CompressedBatch;
pub mod CompressionAlgorithm;
pub mod CompressionInfo;
pub mod CompressionLevel;
pub mod Compressor;
