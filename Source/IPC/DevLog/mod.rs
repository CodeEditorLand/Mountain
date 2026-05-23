//! # DevLog - Tag-filtered development logging
//!
//! Tag-gated logging used across Mountain. Controlled by the
//! `Trace` env var: `Trace=vfs,ipc` for selective tags,
//! `Trace=all` for everything, `Trace=short` for the
//! everything-but-firehose preset (path aliasing + dedupe).
//! Mirror to a session log file when `Record=1` (or when
//! `Trace` is set in a debug build).
//!
//! Layout: every public Fn/Struct lives in its own sibling
//! file. The two macros (`dev_log!`, `otel_span!`) live here
//! so `#[macro_export]` puts them at the crate root and the
//! callsite spelling stays `dev_log!("ipc", "…")`.

pub mod AliasPath;

pub mod AppDataPrefix;

pub mod DebugOnce;

pub mod DedupState;

pub mod EmitOTLPSpan;

pub mod FlushDedup;

pub mod InitEager;

pub mod IsBenignEnoent;

pub mod IsEnabled;

pub mod IsShort;

pub mod NowNano;

pub mod SessionTimestamp;

pub mod WriteToFile;

/// Tag-gated dev log. Compiled out in release builds.
///
/// Under `Trace=short` aliases the long Tauri app-data prefix
/// to `$APP` and collapses consecutive duplicates with a
/// `(xN)` tail. The body is fully gated on
/// `cfg!(debug_assertions)` so release builds get zero runtime
/// cost (LLVM dead-codes the format / IsEnabled / file-sink
/// path).
#[macro_export]
macro_rules! dev_log {

	($Tag:expr, $($Arg:tt)*) => {

		if cfg!(debug_assertions) && $crate::IPC::DevLog::IsEnabled::Fn($Tag) {

			let RawMessage = format!($($Arg)*);

			let TagUpper = $Tag.to_uppercase();

			if $crate::IPC::DevLog::IsShort::Fn() {

				let Aliased = $crate::IPC::DevLog::AliasPath::Fn(&RawMessage);

				let Key = format!("{}:{}", TagUpper, Aliased);

				let ShouldPrint = {

					if let Ok(mut State) = $crate::IPC::DevLog::DedupState::DEDUP.lock() {

						if State.LastKey == Key {

							State.Count += 1;

							false
						} else {

							let PrevCount = State.Count;

							let HadPrev = !State.LastKey.is_empty();

							State.LastKey = Key;

							State.Count = 1;

							if HadPrev && PrevCount > 1 {

								let Tail = format!("  (x{})", PrevCount);

								eprintln!("{}", Tail);

								$crate::IPC::DevLog::WriteToFile::Fn(&Tail);
							}

							true
						}
					} else {

						true
					}
				};

				if ShouldPrint {

					let Formatted = format!("[DEV:{}] {}", TagUpper, Aliased);

					eprintln!("{}", Formatted);

					$crate::IPC::DevLog::WriteToFile::Fn(&Formatted);
				}
			} else {

				let Formatted = format!("[DEV:{}] {}", TagUpper, RawMessage);

				eprintln!("{}", Formatted);

				$crate::IPC::DevLog::WriteToFile::Fn(&Formatted);
			}
		}
	};
}

/// Convenience macro: emit an OTLP span for an IPC handler.
/// Usage: `otel_span!("file:readFile", StartNano, &[("path", &Path)]);`
#[macro_export]
macro_rules! otel_span {
	($Name:expr, $Start:expr, $Attrs:expr) => {
		$crate::IPC::DevLog::EmitOTLPSpan::Fn($Name, $Start, $crate::IPC::DevLog::NowNano::Fn(), $Attrs)
	};

	($Name:expr, $Start:expr) => {
		$crate::IPC::DevLog::EmitOTLPSpan::Fn($Name, $Start, $crate::IPC::DevLog::NowNano::Fn(), &[])
	};
}
