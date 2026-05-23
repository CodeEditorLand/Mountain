//! # PostHog Plugin
//!
//! Debug-only PostHog analytics integration. Captures lifecycle events,
//! IPC commands, and errors during dev runs; compile-time gated to
//! `cfg(debug_assertions)` so release builds drop the entire stack.
//!
//! Layout (one export per file, file name = identity):
//! - `Initialize::Fn` - boot the global ingestion client.
//! - `CaptureEvent::Fn` - generic event emitter with `$app` / `$component`
//!   standard props.
//! - `CaptureError::Fn` - emit under `land:mountain:error` with tag + message.
//! - `CaptureIPC::Fn` - emit under `land:mountain:ipc:invoke` with method name.
//! - `CaptureHandler::Fn` - emit under `land:mountain:handler:complete` with
//!   `feature` + `duration_ms` + `ok`. Powers the Feature Parity dashboard's
//!   Node-vs-Rust handler latency comparison.
//!
//! Module-private helpers:
//! - `Constants` - `Authorize` / `Beam` / `Report` / `Brand` baked from
//!   `.env.Land.PostHog`.
//! - `Client::CLIENT` - `OnceLock<posthog_rs::Client>` singleton.
//! - `DistinctId::Fn` - pinned-or-derived dev distinct id.
//! - `CaptureAllowed::Fn` - `debug_assertions` && `Report != off`.

pub mod CaptureError;

pub mod CaptureEvent;

pub mod CaptureHandler;

pub mod CaptureIPC;

pub mod HydrateRuntimeEnvironment;

pub mod Initialize;

pub(crate) mod CaptureAllowed;

pub(crate) mod Client;

pub(crate) mod Constants;

pub(crate) mod DistinctId;
