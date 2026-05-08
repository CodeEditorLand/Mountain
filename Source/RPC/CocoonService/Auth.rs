#![allow(non_snake_case)]

//! Authentication domain handlers for `CocoonService`. Two gRPC entry
//! points: `GetAuthenticationSession::Fn` and
//! `RegisterAuthenticationProvider::Fn`.

pub mod GetAuthenticationSession;

pub mod RegisterAuthenticationProvider;
