//! Vine client - callers use `::Vine::Client::X::Fn(...)` directly.
//! Mountain's 12 wrapper files were removed; their module declarations are
//! gone. Only the dead stubs (PublishNotification, PublishNotificationFromMux,
//! Shared) are kept so the module path compiles for the no-op bodies.

pub(crate) mod PublishNotification;

pub(crate) mod PublishNotificationFromMux;

pub(crate) mod Shared;
