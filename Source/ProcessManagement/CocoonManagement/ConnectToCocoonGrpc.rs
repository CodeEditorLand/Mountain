//! Dial Cocoon's gRPC server with exponential backoff and child-exit
//! detection. Abandons immediately (with the real exit status) if the Node
//! child dies, and gives up once the total connect budget is exhausted.

use std::time::Duration;

use CommonLibrary::Error::CommonError::CommonError;
use tokio::{process::Child, time::sleep};

pub(crate) async fn Fn(SideCarIdentifier:&str, ChildProcess:&mut Child) -> Result<(), CommonError> {
	// Establish Vine connection to Cocoon with exponential-backoff
	// retry + child-exit detection.
	//
	// Prior policy was 20 × 1000 ms fixed poll. Under healthy timing
	// (Cocoon binds at 150-600 ms) that wasted ~400 ms of idle time
	// every boot; under a genuinely dead Cocoon (import error, killed
	// process, stale bundle) it burned 20 full seconds before giving
	// up with a generic "is Cocoon running?" hint.
	//
	// New policy:
	//   - Initial 50 ms sleep, doubled per attempt up to a 2 s ceiling.
	//   - Hard 20 s total-budget (unchanged) so the overall failure ceiling doesn't
	//     regress for pathological slow-boot hardware.
	//   - Before each sleep, poll `ChildProcess.try_wait()`: if Node has exited,
	//     abandon the loop immediately with the exit status embedded in the error -
	//     no point retrying against a dead process, and the exit code usually
	//     reveals the import failure (1 = unhandled exception, 13 = invalid
	//     module).
	let GRPCAddress = format!("127.0.0.1:{}", super::COCOON_GRPC_PORT);

	crate::dev_log!(
		"cocoon",
		"[CocoonManagement] Connecting to Cocoon gRPC at {} (exponential backoff, budget={}ms)...",
		GRPCAddress,
		super::GRPC_CONNECT_BUDGET_MS
	);

	let ConnectStart = tokio::time::Instant::now();

	let mut CurrentDelayMs:u64 = super::GRPC_CONNECT_INITIAL_MS;

	let mut ConnectAttempt = 0u32;

	loop {
		ConnectAttempt += 1;

		crate::dev_log!(
			"grpc",
			"connecting to Cocoon at {} (attempt {}, elapsed={}ms)",
			GRPCAddress,
			ConnectAttempt,
			ConnectStart.elapsed().as_millis()
		);

		match ::Vine::Client::ConnectToSideCar::Fn(SideCarIdentifier.to_string(), GRPCAddress.clone()).await {
			Ok(()) => {
				crate::dev_log!(
					"grpc",
					"connected to Cocoon on attempt {} (elapsed={}ms)",
					ConnectAttempt,
					ConnectStart.elapsed().as_millis()
				);

				break;
			},

			Err(Error) => {
				// Check if the Node child has already died. If yes,
				// there is no point waiting any longer - report the
				// real exit status so the dev log points at the real
				// failure (import error, crash, oom kill) instead of
				// the abstract "connect refused" message.
				match ChildProcess.try_wait() {
					Ok(Some(ExitStatus)) => {
						let ExitCode = ExitStatus.code().unwrap_or(-1);

						crate::dev_log!(
							"grpc",
							"attempt {} aborted: Cocoon Node process exited with code={} after {}ms - stderr above \
							 (if any) explains why",
							ConnectAttempt,
							ExitCode,
							ConnectStart.elapsed().as_millis()
						);

						return Err(CommonError::IPCError {
							Description:format!(
								"Cocoon spawned but exited with code {} before Mountain could connect. See \
								 `[DEV:COCOON] warn: [Cocoon stderr] …` lines above for the Node-side error - \
								 typically a missing bundle (\"Cannot find module …\") or an ESM/CJS import drift \
								 after a partial build.",
								ExitCode
							),
						});
					},

					Ok(None) => { /* still running, keep trying */ },

					Err(WaitErr) => {
						// try_wait() itself failed; this is rare
						// (would imply a kernel-level issue). Surface
						// it but keep trying - the dial may still
						// succeed on the next attempt.
						crate::dev_log!("grpc", "warn: try_wait on Cocoon child failed: {} (continuing)", WaitErr);
					},
				}

				let Elapsed = ConnectStart.elapsed().as_millis() as u64;

				if Elapsed >= super::GRPC_CONNECT_BUDGET_MS {
					crate::dev_log!(
						"grpc",
						"attempt {} timed out (budget {}ms exhausted): {}",
						ConnectAttempt,
						super::GRPC_CONNECT_BUDGET_MS,
						Error
					);

					return Err(CommonError::IPCError {
						Description:format!(
							"Failed to connect to Cocoon gRPC at {} after {} attempts over {}ms: {} (is Cocoon \
							 running? check `[DEV:COCOON]` log lines for stderr, or re-run with the debug-electron \
							 build profile if the bundle is stale)",
							GRPCAddress,
							ConnectAttempt,
							super::GRPC_CONNECT_BUDGET_MS,
							Error
						),
					});
				}

				crate::dev_log!(
					"grpc",
					"attempt {} pending (Cocoon still booting): {}, backing off {}ms",
					ConnectAttempt,
					Error,
					CurrentDelayMs
				);

				sleep(Duration::from_millis(CurrentDelayMs)).await;

				// Exponential ramp with a 2 s ceiling. Doubling keeps
				// the common case fast (4 attempts cover the first
				// 750 ms) and the cold-boot case bounded.
				CurrentDelayMs = (CurrentDelayMs * 2).min(super::GRPC_CONNECT_MAX_DELAY_MS);
			},
		}
	}

	Ok(())
}
