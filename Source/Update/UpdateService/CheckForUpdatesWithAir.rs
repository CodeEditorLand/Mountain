#![allow(non_snake_case)]

//! Mode-aware update check. Routes to `CheckForUpdates::Fn` (Tauri) or
//! `CheckForUpdatesViaAir::Fn` (Air gRPC) per `UpdateMode::Enum`.
//!
//! Two cfg-gated copies keep the Air-feature build optional: the Air variant
//! pulls in `tonic` + `AirLibrary`; the no-Air variant compiles even when
//! those crates are absent and rejects `ForceAir` at runtime.

use std::sync::Arc;

#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use CommonLibrary::Error::CommonError::CommonError;
use tauri::AppHandle;

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime as Runtime,
	Update::UpdateService::{CheckForUpdates, UpdateMode},
	dev_log,
};
#[cfg(feature = "AirIntegration")]
use crate::Update::UpdateService::{CheckForUpdatesViaAir, IsAirAvailable};

#[cfg(not(feature = "AirIntegration"))]
pub async fn Fn(
	ApplicationHandle:AppHandle,
	RunTime:Arc<Runtime>,
	NotifyNoUpdate:bool,
	_AirClient:Option<()>,
	Mode:UpdateMode::Enum,
) -> Result<(), CommonError> {
	if matches!(Mode, UpdateMode::Enum::ForceAir) {
		return Err(CommonError::Configuration {
			Message:"Air integration is not enabled. Build with `--features AirIntegration` to use ForceAir mode."
				.to_string(),
		});
	}
	dev_log!("update", "[UpdateService] Using Tauri updater (Air integration disabled)");
	CheckForUpdates::Fn(ApplicationHandle, RunTime, NotifyNoUpdate).await
}

#[cfg(feature = "AirIntegration")]
pub async fn Fn(
	ApplicationHandle:AppHandle,
	RunTime:Arc<Runtime>,
	NotifyNoUpdate:bool,
	AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
	Mode:UpdateMode::Enum,
) -> Result<(), CommonError> {
	match Mode {
		UpdateMode::Enum::ForceAir => {
			let AirRef = AirClient.as_ref().ok_or_else(|| {
				CommonError::Configuration { Message:"ForceAir mode requires a valid AirClient".to_string() }
			})?;
			CheckForUpdatesViaAir::Fn(ApplicationHandle, RunTime, NotifyNoUpdate, AirRef).await
		},
		UpdateMode::Enum::ForceTauri => CheckForUpdates::Fn(ApplicationHandle, RunTime, NotifyNoUpdate).await,
		UpdateMode::Enum::AutoDetect => {
			if let Some(AirRef) = &AirClient {
				if IsAirAvailable::Fn(AirRef).await {
					return CheckForUpdatesViaAir::Fn(ApplicationHandle, RunTime, NotifyNoUpdate, AirRef).await;
				}
				dev_log!(
					"update",
					"warn: [UpdateService] Air client provided but unhealthy - falling back to Tauri"
				);
			}
			CheckForUpdates::Fn(ApplicationHandle, RunTime, NotifyNoUpdate).await
		},
	}
}
