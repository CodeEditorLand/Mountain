//! Vine client - callers use `::Vine::Client::X::Fn(...)` directly.

/// Checksidecarhealth module.
pub mod CheckSideCarHealth;

/// Connecttosidecar module.
pub mod ConnectToSideCar;

/// Disconnectfromsidecar module.
pub mod DisconnectFromSideCar;

/// Isclientconnected module.
pub mod IsClientConnected;

/// Isshuttingdown module.
pub mod IsShuttingDown;

/// Markshutdown module.
pub mod MarkShutdown;

/// Notificationframe module.
pub mod NotificationFrame;

/// Publishnotification module.
pub mod PublishNotification;

/// Publishnotificationfrommux module.
pub mod PublishNotificationFromMux;

/// Sendnotification module.
pub mod SendNotification;

/// Sendrequest module.
pub mod SendRequest;

/// Shared module.
pub mod Shared;

/// Subscribenotifications module.
pub mod SubscribeNotifications;

/// Subscribercount module.
pub mod SubscriberCount;

/// Tryconnectsingle module.
pub mod TryConnectSingle;

/// Waitforclientconnection module.
pub mod WaitForClientConnection;
