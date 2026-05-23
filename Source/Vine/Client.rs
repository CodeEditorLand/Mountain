//! Vine client - thread-safe gRPC client for a Cocoon sidecar process.
//! Pool of `CocoonClient` connections keyed by identifier, automatic
//! reconnect with exponential back-off, per-connection health metadata,
//! per-call timeouts, message-size validation, and a broadcast fan-out
//! of every observed notification.
//!
//! Atomized layout (one entry-point per file):
//!   - `MarkShutdown::Fn` / `IsShuttingDown::Fn` - process-wide flag.
//!   - `NotificationFrame::Struct` - broadcast payload.
//!   - `SubscribeNotifications::Fn` / `SubscriberCount::Fn` - fan-out access.
//!   - `ConnectToSideCar::Fn` / `DisconnectFromSideCar::Fn` - pool lifecycle.
//!     Driven by `TryConnectSingle::Fn` (single attempt).
//!   - `IsClientConnected::Fn` / `WaitForClientConnection::Fn` -
//!     boot-race-friendly readiness checks.
//!   - `CheckSideCarHealth::Fn` - pool + metadata health summary.
//!   - `SendRequest::Fn` / `SendNotification::Fn` - wire dispatch with optional
//!     streaming-multiplexer fast path under `LAND_VINE_STREAMING=1`.
//!   - `PublishNotification::Fn` (private) and `PublishNotificationFromMux::Fn`
//!     (`pub(crate)`) - broadcast publishers.
//!   - `Shared` - module-private state (statics, helpers, constants).

pub mod CheckSideCarHealth;

pub mod ConnectToSideCar;

pub mod DisconnectFromSideCar;

pub mod IsClientConnected;

pub mod IsShuttingDown;

pub mod MarkShutdown;

pub mod NotificationFrame;

pub mod PublishNotificationFromMux;

pub mod SendNotification;

pub mod SendRequest;

pub mod SubscribeNotifications;

pub mod SubscriberCount;

pub mod WaitForClientConnection;

pub(crate) mod PublishNotification;

pub(crate) mod Shared;

pub(crate) mod TryConnectSingle;
