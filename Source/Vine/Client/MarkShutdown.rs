//! Flip the global Vine-client shutdown flag. Called from
//! `RunTime::Shutdown::ShutdownCocoonWithRetry` immediately before
//! `HardKillCocoon` so any inflight notification attempted after the
//! SIGKILL window returns silently with `Ok(())` instead of logging a
//! `Connection refused` error.

pub fn Fn() { ::Vine::Client::MarkShutdown::Fn() }
