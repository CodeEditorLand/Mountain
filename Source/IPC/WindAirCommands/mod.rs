//! # Wind ↔ Air delegation commands
//!
//! Tauri commands the Wind frontend invokes to delegate
//! background work (updates, downloads, auth, indexing,
//! search, metrics) to the Air gRPC daemon. Each command +
//! DTO lives in its own sibling file; the wire-bound names
//! (`CheckForUpdates`, `DownloadUpdate`, etc.) are preserved
//! so the front-end `invoke()` calls don't change.

pub mod AirClientWrapper;

pub mod AirMetricsDTO;

pub mod AirServiceStatusDTO;

pub mod ApplyUpdate;

pub mod AuthResponseDTO;

pub mod AuthenticateUser;

pub mod CheckForUpdates;

pub mod DownloadFile;

pub mod DownloadResultDTO;

pub mod DownloadUpdate;

pub mod FileResultDTO;

pub mod GetAirAddress;

pub mod GetAirMetrics;

pub mod GetAirStatus;

pub mod GetOrCreateAirClient;

pub mod IndexFiles;

pub mod IndexResultDTO;

pub mod RegisterWindAirCommands;

pub mod SearchFiles;

pub mod SearchResultsDTO;

pub mod UpdateInfoDTO;
