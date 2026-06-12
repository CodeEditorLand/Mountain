//! Subscribe to the global notification fan-out. Each call returns a
//! fresh receiver that observes every notification fanned out AFTER
//! subscribe time (broadcast semantics; no historical replay). Drop the
//! receiver to unsubscribe.

use crate::Vine::Client::NotificationFrame;

/// Public entry point for this module.
pub fn Fn() -> tokio::sync::broadcast::Receiver<NotificationFrame::Struct> {
	::Vine::Client::SubscribeNotifications::Fn()
}
