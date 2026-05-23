//! # Binary::Shutdown
//!
//! Graceful shutdown sequences for the Mountain binary.
//! Called from the Tauri `on_window_event` / `CloseRequested` handler
//! to drain the Tokio scheduler and terminate the async runtime cleanly
//! before the process exits.

/// Drain pending tasks and shut down the Tokio runtime.
pub mod RuntimeShutdown;

/// Flush the task scheduler queue and cancel outstanding timers.
pub mod SchedulerShutdown;
