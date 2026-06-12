//! Vine client - callers use `::Vine::Client::X::Fn(...)` directly.

/// Checks the health of a sidecar connection.
pub mod CheckSideCarHealth;

/// Establishes a gRPC connection to a sidecar.
pub mod ConnectToSideCar;

/// Disconnects an active gRPC connection to a sidecar.
pub mod DisconnectFromSideCar;

/// Reports whether a sidecar client is connected.
pub mod IsClientConnected;

/// Reports whether the client is marked for shutdown.
pub mod IsShuttingDown;

/// Marks the client for shutdown, preventing new requests.
pub mod MarkShutdown;

/// Encapsulates a single notification frame for the multiplexer.
pub mod NotificationFrame;

/// Publishes a notification to the broadcast channel.
pub mod PublishNotification;

/// Publishes a notification from the streaming multiplexer.
pub mod PublishNotificationFromMux;

/// Sends a fire-and-forget notification to the sidecar.
pub mod SendNotification;

/// Sends a request to the sidecar and awaits a response.
pub mod SendRequest;

/// Shared client state (connection pool, broadcast, shutdown flag).
pub mod Shared;

/// Subscribes to the notification broadcast channel.
pub mod SubscribeNotifications;

/// Reports the number of active broadcast subscribers.
pub mod SubscriberCount;

/// Attempts a single gRPC connection without retry.
pub mod TryConnectSingle;

/// Blocks until the sidecar client establishes a connection.
pub mod WaitForClientConnection;
