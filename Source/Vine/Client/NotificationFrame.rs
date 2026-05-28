//! One observed notification frame fanned out from `SendNotification`
//! (or, once the streaming-channel multiplexer is live, from
//! `Multiplexer`). Subscribers consume frames from the broadcast channel
//! managed by `Shared::NOTIFICATION_BROADCAST`.

/// Canonical frame shape. See [`::Vine::Client::NotificationFrame::Struct`].
pub type Struct = ::Vine::Client::NotificationFrame::Struct;
