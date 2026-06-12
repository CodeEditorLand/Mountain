//! Authentication domain handlers for `CocoonService`. Two gRPC entry
//! points: `GetAuthenticationSession::Fn` and
//! `RegisterAuthenticationProvider::Fn`.
/// GetAuthenticationSession handler: retrieves an authentication session for
/// the given provider.
pub mod GetAuthenticationSession;

/// RegisterAuthenticationProvider handler: registers an authentication
/// provider.
pub mod RegisterAuthenticationProvider;
