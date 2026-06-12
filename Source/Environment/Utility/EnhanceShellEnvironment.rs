//! macOS / Linux GUI launches (Finder double-click, Dock, Spotlight,
//! `open <bundle>.app`) hand the app a minimal environment:
//! `PATH=/usr/bin:/bin:/usr/sbin:/sbin`, no `NVM_DIR`, no `HOMEBREW_PREFIX`,
//! no `JAVA_HOME`, …
//!
//! That breaks every child process Mountain or its extensions spawn:
//! - Cocoon's `node` binary can't find Homebrew installs (`/opt/homebrew/bin`,
//!   `/usr/local/bin`).
//! - Language servers (rust-analyzer, gopls, pyright) probe `PATH` and fail to
//!   launch.
//! - Git extensions invoking `git` fall back to `/usr/bin/git` (Apple's ancient
//!   stock copy) instead of the Homebrew one.
//!
//! VS Code, Atom, and most other Electron editors solve this by spawning
//! the user's interactive shell with `-ilc env` once at boot and merging
//! the result into the process environment. We do the same here.
//!
//! Skipped when:
//! - The launcher is already a TTY (the user invoked from a terminal - PATH is
//!   already correct).
//! - `Walk=0` (matches the existing knob users may rely on).
//! - The shell probe fails or times out (best-effort; never fatal).

use std::time::Duration;

/// In-flight `$SHELL -ilc env` capture started by [`Begin`]. The shell
/// runs concurrently with the rest of the pre-runtime boot work (keyring
/// init, log-sink open, `.env.Land` probing); [`Probe::Finish`] joins it
/// with a bounded deadline and merges the result into `std::env`.
pub struct Probe {
	Child:std::process::Child,

	Deadline:std::time::Instant,
}

/// Run `$SHELL -ilc env` and merge novel keys into `std::env`. Existing
/// values win - never clobber an env var the parent process explicitly
/// set (especially `PATH` if the user passed one). Caller is expected
/// to invoke this exactly once during boot, before any child process
/// is spawned.
pub fn Fn() {
	if let Some(Pending) = Begin() {
		Pending.Finish();
	}
}

/// Spawn the interactive-shell probe without waiting on it. Returns
/// `None` when the probe is unnecessary (TTY launch) or the spawn
/// itself fails. The deadline is anchored at spawn time, so any boot
/// work the caller performs before `Finish()` counts toward the
/// shell's budget instead of extending it.
pub fn Begin() -> Option<Probe> {
	// TTY = launched from terminal = already has the user's shell env.
	if IsTty() {
		return None;
	}

	let Shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

	// `-i` (interactive) loads `~/.zshrc` / `~/.bashrc` where users
	// typically extend PATH. `-l` (login) loads `~/.zprofile` /
	// `~/.bash_profile` where Homebrew, NVM, and similar set their
	// roots. `-c env` prints every var the shell knows.
	let Output = std::process::Command::new(&Shell)
		.args(["-ilc", "env"])
		.stdin(std::process::Stdio::null())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::null())
		.spawn();

	match Output {
		// Hard cap so a misbehaving rc-file (network call in `.zshrc`,
		// blocking `read`) doesn't stall boot. 800 ms from spawn keeps
		// the worst-case main-thread stall under one frame of perceived
		// launch delay; the merge must stay on the single-threaded boot
		// path because `set_var` is unsound once Tokio workers exist.
		Ok(Child) => Some(Probe { Child, Deadline:std::time::Instant::now() + Duration::from_millis(800) }),

		Err(_) => None,
	}
}

impl Probe {
	/// Wait (bounded by the deadline set at [`Begin`]) for the shell to
	/// exit, then merge novel keys into `std::env`. Must run on the
	/// single-threaded boot path before the Tokio runtime is built -
	/// `set_var` races `getenv` from any other live thread.
	pub fn Finish(mut self) {
		loop {
			match self.Child.try_wait() {
				Ok(Some(_)) => break,

				Ok(None) => {
					if std::time::Instant::now() >= self.Deadline {
						let _ = self.Child.kill();

						let _ = self.Child.wait();

						return;
					}

					std::thread::sleep(Duration::from_millis(20));
				},

				Err(_) => return,
			}
		}

		let StdoutBytes = match self.Child.wait_with_output() {
			Ok(O) => O.stdout,

			Err(_) => return,
		};

		let Text = match String::from_utf8(StdoutBytes) {
			Ok(S) => S,

			Err(_) => return,
		};

		for Line in Text.lines() {
			let Some((Key, Value)) = Line.split_once('=') else { continue };

			let Key = Key.trim();

			if Key.is_empty() || !IsPortableEnvName(Key) {
				continue;
			}

			// PATH is special: we only reach this point because IsTty() was
			// false, meaning the process was launched from Finder/Dock/launchd
			// with PATH=/usr/bin:/bin:/usr/sbin:/sbin.  That minimal value
			// is NOT the user's intentional PATH - always let the shell
			// replace it so git, node, language servers, etc. are all found.
			// For every other var, preserve any explicit value the user set
			// (e.g. `FOO=bar open /Applications/X.app`).
			if Key != "PATH" && std::env::var_os(Key).is_some() {
				continue;
			}

			// SAFETY: pre-window, single-threaded boot path. set_var is
			// safe at this point. Mountain's other modules read env
			// through `std::env::var` snapshots after this returns.
			unsafe { std::env::set_var(Key, Value) };
		}
	}
}

fn IsTty() -> bool {
	// `IsTerminal` (stable since Rust 1.70) wraps platform isatty
	// without pulling in libc. Stdin is the right fd to probe -
	// Mountain redirects stdout/stderr to its own logger, so those
	// always look "non-tty" even from a real terminal.
	use std::io::IsTerminal;

	std::io::stdin().is_terminal()
}

/// Reject keys with characters outside the portable POSIX set so a
/// hostile rc-file can't sneak shell metacharacters into our env via a
/// crafted `Key=` line. Standard env-var names are
/// `[A-Za-z_][A-Za-z0-9_]*`; anything else is dropped silently.
fn IsPortableEnvName(Name:&str) -> bool {
	let mut Chars = Name.chars();

	match Chars.next() {
		Some(C) if C.is_ascii_alphabetic() || C == '_' => {},

		_ => return false,
	}

	Chars.all(|C| C.is_ascii_alphanumeric() || C == '_')
}
