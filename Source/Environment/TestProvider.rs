
//! # TestProvider (Environment)
//!
//! `TestController` impl for `MountainEnvironment`. Hosts the
//! controller registry and routes test runs through proxied sidecars
//! (extension-provided test frameworks). Native Rust controllers are
//! not yet supported - they short-circuit to `Skipped`.
//!
//! Layout (one export per file, file name = identity):
//! - `TestControllerState::Struct` - per-controller registration.
//! - `TestRunStatus::Enum` - Queued / Running / Passed / Failed / Skipped /
//!   Errored.
//! - `TestResult::Struct` - per-test outcome.
//! - `TestRun::Struct` - active test run record.
//! - `TestProviderState::Struct` - aggregate controller + active-runs map, held
//!   inside `ApplicationState` behind a `RwLock`.
//!
//! The trait impl `TestController for MountainEnvironment` and its
//! private helpers stay in this parent file; they are dispatched via
//! the trait, not directly addressable, so they don't need atomic
//! split for navigability.
//!
//! VS Code reference:
//! - `vs/workbench/contrib/testing/common/testService.ts`,
//! - `vs/workbench/contrib/testing/common/testTypes.ts`.

pub mod TestControllerState;

pub mod TestProviderState;

pub mod TestResult;

pub mod TestRun;

pub mod TestRunStatus;

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider, SkyEvent::SkyEvent},
	Testing::TestController::TestController,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::Emitter;
use uuid::Uuid;

use super::MountainEnvironment::MountainEnvironment;
use crate::dev_log;

#[async_trait]
impl TestController for MountainEnvironment {
	async fn RegisterTestController(&self, ControllerId:String, Label:String) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[TestProvider] Registering test controller '{}' with label '{}'",
			ControllerId,
			Label
		);

		let SideCarIdentifier = Some("cocoon-main".to_string());

		let ControllerState = TestControllerState::Struct {
			ControllerIdentifier:ControllerId.clone(),

			Label,

			SideCarIdentifier,

			IsActive:true,

			SupportedTestTypes:vec!["unit".to_string(), "integration".to_string()],
		};

		let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

		StateGuard.Controllers.insert(ControllerId.clone(), ControllerState);

		drop(StateGuard);

		self.ApplicationHandle
			.emit(
				SkyEvent::TestRegistered.AsStr(),
				json!({ "ControllerIdentifier": ControllerId }),
			)
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit test registration event: {}", Error) }
			})?;

		dev_log!(
			"extensions",
			"[TestProvider] Test controller '{}' registered successfully",
			ControllerId
		);

		Ok(())
	}

	async fn RunTests(&self, ControllerIdentifier:String, TestRunRequest:Value) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[TestProvider] Running tests for controller '{}': {:?}",
			ControllerIdentifier,
			TestRunRequest
		);

		let ControllerState = {
			let StateGuard = self.ApplicationState.TestProviderState.read().await;

			StateGuard.Controllers.get(&ControllerIdentifier).cloned().ok_or_else(|| {
				CommonError::TestControllerNotFound { ControllerIdentifier:ControllerIdentifier.clone() }
			})?
		};

		let RunIdentifier = Uuid::new_v4().to_string();

		let TestRunRecord = TestRun::Struct {
			RunIdentifier:RunIdentifier.clone(),

			ControllerIdentifier:ControllerIdentifier.clone(),

			Status:TestRunStatus::Enum::Queued,

			StartedAt:std::time::Instant::now(),

			Results:std::collections::HashMap::new(),
		};

		{
			let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

			StateGuard.ActiveRuns.insert(RunIdentifier.clone(), TestRunRecord);
		}

		self.ApplicationHandle
			.emit(
				SkyEvent::TestRunStarted.AsStr(),
				json!({ "RunIdentifier": RunIdentifier, "ControllerIdentifier": ControllerIdentifier }),
			)
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit test run started event: {}", Error) }
			})?;

		if let Some(SideCarIdentifier) = &ControllerState.SideCarIdentifier {
			Self::RunProxiedTests(self, SideCarIdentifier, &RunIdentifier, TestRunRequest).await?;
		} else {
			dev_log!(
				"extensions",
				"warn: [TestProvider] Native test controllers not yet implemented for '{}'",
				ControllerIdentifier
			);

			let _ = Self::UpdateRunStatus(self, &RunIdentifier, TestRunStatus::Enum::Skipped).await;
		}

		Ok(())
	}
}

impl MountainEnvironment {
	async fn RunProxiedTests(
		&self,

		SideCarIdentifier:&str,

		RunIdentifier:&str,

		TestRunRequest:Value,
	) -> Result<(), CommonError> {
		dev_log!(
			"extensions",
			"[TestProvider] Running proxied tests for run '{}' on sidecar '{}'",
			RunIdentifier,
			SideCarIdentifier
		);

		let _ = Self::UpdateRunStatus(self, RunIdentifier, TestRunStatus::Enum::Running).await;

		let IPCProviderHandle:Arc<dyn IPCProvider> = self.Require();

		let RPCMethod = format!("{}$runTests", ProxyTarget::ExtHostTesting.GetTargetPrefix());

		let RPCParams = json!({ "RunIdentifier": RunIdentifier, "TestRunRequest": TestRunRequest });

		match IPCProviderHandle
			.SendRequestToSideCar(SideCarIdentifier.to_string(), RPCMethod, RPCParams, 300000)
			.await
		{
			Ok(Response) => {
				if let Ok(Results) = serde_json::from_value::<Vec<TestResult::Struct>>(Response) {
					let _ = Self::StoreTestResults(self, RunIdentifier, Results).await;

					let FinalStatus = Self::CalculateRunStatus(self, RunIdentifier).await;

					let _ = Self::UpdateRunStatus(self, RunIdentifier, FinalStatus).await;

					dev_log!(
						"extensions",
						"[TestProvider] Test run '{}' completed with status {:?}",
						RunIdentifier,
						FinalStatus
					);
				} else {
					dev_log!(
						"extensions",
						"error: [TestProvider] Failed to parse test results for run '{}'",
						RunIdentifier
					);

					let _ = Self::UpdateRunStatus(self, RunIdentifier, TestRunStatus::Enum::Errored).await;
				}

				Ok(())
			},

			Err(Error) => {
				dev_log!("extensions", "error: [TestProvider] Failed to run tests: {}", Error);

				let _ = Self::UpdateRunStatus(self, RunIdentifier, TestRunStatus::Enum::Errored).await;

				Err(Error)
			},
		}
	}

	async fn UpdateRunStatus(&self, RunIdentifier:&str, Status:TestRunStatus::Enum) -> Result<(), CommonError> {
		let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

		if let Some(TestRunRecord) = StateGuard.ActiveRuns.get_mut(RunIdentifier) {
			TestRunRecord.Status = Status;

			drop(StateGuard);

			self.ApplicationHandle
				.emit(
					SkyEvent::TestRunStatusChanged.AsStr(),
					json!({ "RunIdentifier": RunIdentifier, "Status": Status }),
				)
				.map_err(|Error| {
					CommonError::IPCError { Description:format!("Failed to emit test status change event: {}", Error) }
				})?;

			Ok(())
		} else {
			Err(CommonError::TestRunNotFound { RunIdentifier:RunIdentifier.to_string() })
		}
	}

	async fn StoreTestResults(&self, RunIdentifier:&str, Results:Vec<TestResult::Struct>) -> Result<(), CommonError> {
		let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

		if let Some(TestRunRecord) = StateGuard.ActiveRuns.get_mut(RunIdentifier) {
			for Result in Results {
				TestRunRecord.Results.insert(Result.TestIdentifier.clone(), Result);
			}

			Ok(())
		} else {
			Err(CommonError::TestRunNotFound { RunIdentifier:RunIdentifier.to_string() })
		}
	}

	async fn CalculateRunStatus(&self, RunIdentifier:&str) -> TestRunStatus::Enum {
		let StateGuard = self.ApplicationState.TestProviderState.read().await;

		if let Some(TestRunRecord) = StateGuard.ActiveRuns.get(RunIdentifier) {
			if TestRunRecord.Results.is_empty() {
				TestRunStatus::Enum::Passed
			} else {
				let HasFailed = TestRunRecord.Results.values().any(|R| R.Status == TestRunStatus::Enum::Failed);

				let HasErrored = TestRunRecord.Results.values().any(|R| R.Status == TestRunStatus::Enum::Errored);

				if HasErrored {
					TestRunStatus::Enum::Errored
				} else if HasFailed {
					TestRunStatus::Enum::Failed
				} else {
					TestRunStatus::Enum::Passed
				}
			}
		} else {
			TestRunStatus::Enum::Errored
		}
	}
}
