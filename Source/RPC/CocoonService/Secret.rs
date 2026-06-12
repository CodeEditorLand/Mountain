//! Secret-storage domain handlers for `CocoonService`.
//! `DeleteSecret::Fn`, `GetSecret::Fn`, `StoreSecret::Fn`.
/// DeleteSecret handler: removes a stored secret.
pub mod DeleteSecret;

/// GetSecret handler: retrieves a stored secret.
pub mod GetSecret;

/// StoreSecret handler: stores a secret securely.
pub mod StoreSecret;
