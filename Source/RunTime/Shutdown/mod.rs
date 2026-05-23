
//! Graceful shutdown of every Mountain service. `Shutdown` orchestrates;
//! `ShutdownWithRecovery` continues across per-service failures.

pub mod DisposeTerminalsSafely;

pub mod FlushPendingOperations;

pub mod SaveApplicationState;

pub mod Shutdown;

pub mod ShutdownCocoonWithRetry;

pub mod ShutdownWithRecovery;
