//! Vine client - callers use `::Vine::Client::X::Fn(...)` directly.

pub mod CheckSideCarHealth;

pub mod ConnectToSideCar;

pub mod DisconnectFromSideCar;

pub mod IsClientConnected;

pub mod IsShuttingDown;

pub mod MarkShutdown;

pub mod NotificationFrame;

pub mod PublishNotification;

pub mod PublishNotificationFromMux;

pub mod SendNotification;

pub mod SendRequest;

pub mod Shared;

pub mod SubscribeNotifications;

pub mod SubscriberCount;

pub mod TryConnectSingle;

pub mod WaitForClientConnection;
