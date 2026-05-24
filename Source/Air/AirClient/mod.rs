//! `AirClient` - atomized.

pub mod Address;
pub mod AirMetrics;
pub mod AirStatus;
pub mod ApplyUpdate;
pub mod Authenticate;
pub mod CheckForUpdates;
pub mod DownloadFile;
pub mod DownloadStreamChunk;
pub mod DownloadUpdate;
pub mod ExtendedFileInfo;
pub mod FileInfo;
pub mod FileResult;
pub mod GetConfiguration;
pub mod GetFileInfo;
pub mod GetMetrics;
pub mod GetResourceUsage;
pub mod GetStatus;
pub mod HealthCheck;
pub mod IndexFiles;
pub mod IndexInfo;
pub mod IsConnected;
pub mod New;
pub mod ResourceUsage;
pub mod SearchFiles;
pub mod SetResourceLimits;
pub mod UpdateConfiguration;
pub mod UpdateInfo;

// AirClient: gRPC client wrapper for the Air daemon service.

pub mod AirMetrics;

pub mod AirStatus;

pub mod DownloadStream;

pub mod DownloadStreamChunk;

pub mod ExtendedFileInfo;

pub mod FileInfo;

pub mod FileResult;

pub mod IndexInfo;

pub mod ResourceUsage;

pub mod UpdateInfo;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};

use crate::dev_log;

/// Default gRPC server address for the Air daemon.
///
/// Port Allocation:
/// - 50051: Mountain Vine server
/// - 50052: Cocoon Vine server (VS Code extension hosting)
/// - 50053: Air Vine server (Air daemon services - authentication, updates, and
///   more)
pub const DEFAULT_AIR_SERVER_ADDRESS:&str = "[::1]:50053";

/// Air gRPC client wrapper that handles connection to the Air daemon service.
/// This provides a clean interface for Mountain to interact with Air's
/// capabilities including update management, authentication, file indexing,
/// and system monitoring.
#[derive(Clone)]
pub struct Struct {
	#[cfg(feature = "AirIntegration")]
	/// The underlying tonic gRPC client wrapped in Arc<Mutex<>> for thread-safe
	/// access
	client:Option<Arc<Mutex<AirServiceClient<Channel>>>>,

	/// Address of the Air daemon
	address:String,
}
