//! Atom I6: pre-boot sweep. TCP-probe the Cocoon gRPC port and kill any
//! stale process still bound to it. Prevents the EADDRINUSE cascade that
//! leaves the extension host in degraded mode when a prior Mountain exited
//! without cleaning up its child.

use crate::dev_log;

/// Behaviour:
/// - If the port answers a TCP connect, assume an owner is listening.
/// - Use `lsof -nP -iTCP:<port> -sTCP:LISTEN -t` (macOS/Linux) to resolve the
///   PID. `lsof` is ubiquitous on macOS/Linux and doesn't require root for
///   local user-owned processes.
/// - SIGTERM first, 500ms grace window, then SIGKILL if still alive.
/// - Logs every step via `dev_log!("cocoon", …)` so the sweep is visible in
///   Mountain.dev.log without parsing stderr.
/// - Best-effort: failures don't abort Mountain boot. A real EADDRINUSE later
///   will surface via Cocoon's own bootstrap error.
/// The TCP probe, `lsof`/`kill` subprocesses, and grace-window sleeps all
/// block the calling thread. Boot calls this from a Tokio worker, so the
/// body runs under `block_in_place` to hand the worker back to the
/// scheduler; outside a multi-thread runtime it runs inline.
pub(crate) fn Fn(Port:u16) {
	match tokio::runtime::Handle::try_current() {
		Ok(Handle) if Handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
			tokio::task::block_in_place(|| SweepBlocking(Port));
		},

		_ => SweepBlocking(Port),
	}
}

fn SweepBlocking(Port:u16) {
	use std::{net::TcpStream, time::Duration};

	let Addr = format!("127.0.0.1:{}", Port);

	// Cheap liveness probe. Timeout is aggressive - zombie ports answer
	// immediately; a clean port is ECONNREFUSED and returns instantly.
	let Probe =
		TcpStream::connect_timeout(&Addr.parse().expect("valid socket addr literal"), Duration::from_millis(200));

	if Probe.is_err() {
		dev_log!("cocoon", "[CocoonSweep] Port {} is clean (no prior listener).", Port);

		return;
	}

	dev_log!(
		"cocoon",
		"[CocoonSweep] Port {} has a listener - attempting to resolve owner via lsof.",
		Port
	);

	// `lsof -nP -iTCP:<port> -sTCP:LISTEN -t` → one PID per line.
	let LsofOutput = std::process::Command::new("lsof")
		.args(["-nP", &format!("-iTCP:{}", Port), "-sTCP:LISTEN", "-t"])
		.output();

	let Output = match LsofOutput {
		Ok(O) => O,

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonSweep] lsof unavailable ({}). Skipping sweep; Cocoon spawn may fail with EADDRINUSE.",
				Error
			);

			return;
		},
	};

	if !Output.status.success() {
		dev_log!("cocoon", "warn: [CocoonSweep] lsof exited non-zero. Skipping sweep.");

		return;
	}

	let Stdout = String::from_utf8_lossy(&Output.stdout);

	let Pids:Vec<i32> = Stdout.lines().filter_map(|L| L.trim().parse::<i32>().ok()).collect();

	if Pids.is_empty() {
		dev_log!(
			"cocoon",
			"warn: [CocoonSweep] Port {} answered but lsof found no LISTEN PID - giving up.",
			Port
		);

		return;
	}

	// Guard against self-kill. Mountain currently binds 50051, not Cocoon's
	// 50052, but belt-and-braces for future refactors.
	let SelfPid = std::process::id() as i32;

	for Pid in Pids {
		if Pid == SelfPid {
			dev_log!(
				"cocoon",
				"warn: [CocoonSweep] Port {} owned by Mountain itself (PID {}); refusing to kill.",
				Port,
				Pid
			);

			continue;
		}

		dev_log!("cocoon", "[CocoonSweep] Killing stale PID {} (SIGTERM).", Pid);

		let _ = std::process::Command::new("kill").arg(Pid.to_string()).status();

		std::thread::sleep(Duration::from_millis(500));

		// Recheck - if still alive, escalate.
		let StillAlive = std::process::Command::new("kill")
			.args(["-0", &Pid.to_string()])
			.status()
			.map(|S| S.success())
			.unwrap_or(false);

		if StillAlive {
			dev_log!("cocoon", "warn: [CocoonSweep] PID {} survived SIGTERM; sending SIGKILL.", Pid);

			let _ = std::process::Command::new("kill").args(["-9", &Pid.to_string()]).status();

			std::thread::sleep(Duration::from_millis(200));
		}

		dev_log!("cocoon", "[CocoonSweep] PID {} reaped.", Pid);
	}
}
