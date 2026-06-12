//! Initialization-domain handlers for `CocoonService`.
//! `CancelOperation::Fn`, `InitialHandshake::Fn`, `InitExtensionHost::Fn`.
/// CancelOperation handler: cancels a pending operation by its cancellation
/// token.
pub mod CancelOperation;

/// InitExtensionHost handler: initializes a new extension host connection.
pub mod InitExtensionHost;

/// InitialHandshake handler: performs the initial handshake with an extension
/// host.
pub mod InitialHandshake;
