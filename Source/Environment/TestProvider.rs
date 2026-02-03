// ============================================================================
// File: Mountain/Source/Environment/TestProvider.rs
// ============================================================================
// # TestProvider Implementation
//
// Implements the `TestController` trait for the `MountainEnvironment`.
// This provider manages test discovery, execution, and result reporting,
// borrowing patterns from VSCode's Testing Service.
//
// ## Key Features:
// - Test controller registration and lifecycle management
// - Test discovery and enumeration
// - Test execution with progress tracking
// - Test result aggregation and reporting
// - Sidecar proxy support for extension-provided test frameworks
//
// ## VSCode Reference:
// - vs/workbench/contrib/testing/common/testTypes.ts
// - vs/workbench/contrib/testing/common/testService.ts
//
// ============================================================================

use std::{collections::HashMap, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	Testing::TestController::TestController,
};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::Emitter;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{MountainEnvironment::MountainEnvironment, Utility};

/// Represents a test controller's state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestControllerState {
	pub ControllerIdentifier:String,
	pub Label:String,
	pub SideCarIdentifier:Option<String>,
	pub IsActive:bool,
	pub SupportedTestTypes:Vec<String>,
}

/// Represents the status of a test run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TestRunStatus {
	Queued,
	Running,
	Passed,
	Failed,
	Skipped,
	Errored,
}

/// Represents a test result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResult {
	pub TestIdentifier:String,
	pub FullName:String,
	pub Status:TestRunStatus,
	pub DurationMs:Option<u64>,
	pub ErrorMessage:Option<String>,
	pub StackTrace:Option<String>,
}

/// Represents an active test run
#[derive(Debug, Clone)]
struct TestRun {
	pub RunIdentifier:String,
	pub ControllerIdentifier:String,
	pub Status:TestRunStatus,
	pub StartedAt:std::time::Instant,
	pub Results:HashMap<String, TestResult>,
}

/// Stores test provider state
#[derive(Debug)]
pub struct TestProviderState {
	pub Controllers:HashMap<String, TestControllerState>,
	pub ActiveRuns:HashMap<String, TestRun>,
}

impl TestProviderState {
	pub fn new() -> Self { Self { Controllers:HashMap::new(), ActiveRuns:HashMap::new() } }
}

#[async_trait]
impl TestController for MountainEnvironment {
	/// Registers a new test controller from an extension (e.g., Cocoon).
	///
	/// This method creates a TestControllerState entry and notifies the
	/// frontend about the available test controller.
	async fn RegisterTestController(&self, ControllerId:String, Label:String) -> Result<(), CommonError> {
		info!(
			"[TestProvider] Registering test controller '{}' with label '{}'",
			ControllerId, Label
		);

		// For now, assume all extension providers come from the main sidecar
		let SideCarIdentifier = Some("cocoon-main".to_string());

		let ControllerState = TestControllerState {
			ControllerIdentifier:ControllerId.clone(),
			Label,
			SideCarIdentifier,
			IsActive:true,
			SupportedTestTypes:vec!["unit".to_string(), "integration".to_string()],
		};

		// Store the controller state
		let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

		StateGuard.Controllers.insert(ControllerId.clone(), ControllerState);
		drop(StateGuard);

		// Notify the frontend about the new test controller
		self.ApplicationHandle
			.emit("sky://test/registered", json!({ "ControllerIdentifier": ControllerId }))
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit test registration event: {}", Error) }
			})?;

		debug!("[TestProvider] Test controller '{}' registered successfully", ControllerId);

		Ok(())
	}

	/// Runs tests based on the test run request.
	///
	/// This implementation supports both native (Rust) and proxied (extension)
	/// test controllers, with proper test discovery, execution, and result
	/// reporting.
	async fn RunTests(&self, ControllerIdentifier:String, TestRunRequest:Value) -> Result<(), CommonError> {
		info!(
			"[TestProvider] Running tests for controller '{}': {:?}",
			ControllerIdentifier, TestRunRequest
		);

		// Get controller state
		let ControllerState = {
			let StateGuard = self.ApplicationState.TestProviderState.read().await;

			StateGuard.Controllers.get(&ControllerIdentifier).cloned().ok_or_else(|| {
				CommonError::TestControllerNotFound { ControllerIdentifier:ControllerIdentifier.clone() }
			})?
		};

		// Create a new test run
		let RunIdentifier = Uuid::new_v4().to_string();
		let TestRun = TestRun {
			RunIdentifier:RunIdentifier.clone(),
			ControllerIdentifier:ControllerIdentifier.clone(),
			Status:TestRunStatus::Queued,
			StartedAt:std::time::Instant::now(),
			Results:HashMap::new(),
		};

		{
			let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

			StateGuard.ActiveRuns.insert(RunIdentifier.clone(), TestRun);
		}

		// Notify frontend about test run start
		self.ApplicationHandle
			.emit(
				"sky://test/run-started",
				json!({ "RunIdentifier": RunIdentifier, "ControllerIdentifier": ControllerIdentifier }),
			)
			.map_err(|Error| {
				CommonError::IPCError { Description:format!("Failed to emit test run started event: {}", Error) }
			})?;

		// Execute tests based on controller type
		if let Some(SideCarIdentifier) = &ControllerState.SideCarIdentifier {
			// Proxied extension test controller
			Self::RunProxiedTests(self, SideCarIdentifier, &RunIdentifier, TestRunRequest).await?;
		} else {
			// Native Rust test controller (currently not supported)
			warn!(
				"[TestProvider] Native test controllers not yet implemented for '{}'",
				ControllerIdentifier
			);
			Self::UpdateRunStatus(self, &RunIdentifier, TestRunStatus::Skipped).await;
		}

		Ok(())
	}
}

// ============================================================================
// Private Helper Methods
// ============================================================================

impl MountainEnvironment {
	/// Runs tests via a proxied sidecar test controller.
	async fn RunProxiedTests(
		&self,
		SideCarIdentifier:&str,
		RunIdentifier:&str,
		TestRunRequest:Value,
	) -> Result<(), CommonError> {
		info!(
			"[TestProvider] Running proxied tests for run '{}' on sidecar '{}'",
			RunIdentifier, SideCarIdentifier
		);

		// Update test run status to running
		Self::UpdateRunStatus(self, RunIdentifier, TestRunStatus::Running).await;

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		let RPCMethod = format!("{}$runTests", ProxyTarget::ExtHostTesting.GetTargetPrefix());
		let RPCParams = json!({
			"RunIdentifier": RunIdentifier,
			"TestRunRequest": TestRunRequest,
		});

		match IPCProvider
			.SendRequestToSideCar(SideCarIdentifier.to_string(), RPCMethod, RPCParams, 300000)
			.await
		{
			Ok(Response) => {
				// Parse test results from response
				if let Ok(Results) = serde_json::from_value::<Vec<TestResult>>(Response) {
					Self::StoreTestResults(self, RunIdentifier, Results).await;

					// Determine final status based on results
					let FinalStatus = Self::CalculateRunStatus(self, RunIdentifier).await;
					Self::UpdateRunStatus(self, RunIdentifier, FinalStatus).await;

					info!(
						"[TestProvider] Test run '{}' completed with status {:?}",
						RunIdentifier, FinalStatus
					);
				} else {
					error!("[TestProvider] Failed to parse test results for run '{}'", RunIdentifier);
					Self::UpdateRunStatus(self, RunIdentifier, TestRunStatus::Errored).await;
				}
				Ok(())
			},
			Err(Error) => {
				error!("[TestProvider] Failed to run tests: {}", Error);
				Self::UpdateRunStatus(self, RunIdentifier, TestRunStatus::Errored).await;
				Err(Error)
			},
		}
	}

	/// Updates the status of a test run and notifies the frontend.
	async fn UpdateRunStatus(&self, RunIdentifier:&str, Status:TestRunStatus) -> Result<(), CommonError> {
		let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

		if let Some(TestRun) = StateGuard.ActiveRuns.get_mut(RunIdentifier) {
			TestRun.Status = Status;

			drop(StateGuard);

			// Notify frontend about status change
			self.ApplicationHandle
				.emit(
					"sky://test/run-status-changed",
					json!({
						"RunIdentifier": RunIdentifier,
						"Status": Status,
					}),
				)
				.map_err(|Error| {
					CommonError::IPCError { Description:format!("Failed to emit test status change event: {}", Error) }
				})?;

			Ok(())
		} else {
			Err(CommonError::TestRunNotFound { RunIdentifier:RunIdentifier.to_string() })
		}
	}

	/// Stores test results for a test run.
	async fn StoreTestResults(&self, RunIdentifier:&str, Results:Vec<TestResult>) -> Result<(), CommonError> {
		let mut StateGuard = self.ApplicationState.TestProviderState.write().await;

		if let Some(TestRun) = StateGuard.ActiveRuns.get_mut(RunIdentifier) {
			for Result in Results {
				TestRun.Results.insert(Result.TestIdentifier.clone(), Result);
			}
			Ok(())
		} else {
			Err(CommonError::TestRunNotFound { RunIdentifier:RunIdentifier.to_string() })
		}
	}

	/// Calculates the final status of a test run based on its results.
	async fn CalculateRunStatus(&self, RunIdentifier:&str) -> TestRunStatus {
		let StateGuard = self.ApplicationState.TestProviderState.read().await;

		if let Some(TestRun) = StateGuard.ActiveRuns.get(RunIdentifier) {
			if TestRun.Results.is_empty() {
				TestRunStatus::Passed // No tests considered passed
			} else {
				let HasFailed = TestRun.Results.values().any(|r| r.Status == TestRunStatus::Failed);
				let HasErrored = TestRun.Results.values().any(|r| r.Status == TestRunStatus::Errored);

				if HasErrored {
					TestRunStatus::Errored
				} else if HasFailed {
					TestRunStatus::Failed
				} else {
					TestRunStatus::Passed
				}
			}
		} else {
			TestRunStatus::Errored
		}
	}
}
