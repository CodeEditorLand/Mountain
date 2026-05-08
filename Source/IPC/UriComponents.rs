#![allow(non_snake_case, dead_code)]

//! # UriComponents - VS Code marshalling helpers
//!
//! Centralised builders for VS Code `UriComponents` payloads. The
//! renderer's reviver (`_transformIncomingURIs` in `uriIpc.ts`) walks
//! every response object and only calls `URI.revive()` on nested
//! objects tagged with `$mid === MarshalledId.Uri (= 1)`. An untagged
//! `UriComponents` reaches callers as a plain bag - `uri.with is not a
//! function`, `uri.fsPath` undefined - and the sidebar / icon loader /
//! `joinPath` chain silently breaks.
//!
//! Layout (one export per file, file name = identity):
//! - `MID_URI::VALUE` - the magic marshalling constant.
//! - `StampMidUri::Fn` - tag a `Value::Object` with `$mid: 1`.
//! - `FromFilePath::Fn` - `file://`-scheme builder from an absolute path.
//! - `FromUrl::Fn` - generic-scheme builder from a URL string.
//! - `Normalize::Fn` - accept string / object / missing and return a tagged URI
//!   bag.

pub mod FromFilePath;

pub mod FromUrl;

pub mod MID_URI;

pub mod Normalize;

pub mod StampMidUri;
