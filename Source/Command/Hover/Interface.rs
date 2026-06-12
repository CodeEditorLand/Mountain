//! # Hover Interface
//!
//! Type definitions for the Hover language feature. LSP-shaped DTOs.
//!
//! Layout (one export per file, file name = identity):
//! - `Position::Struct` - zero-based line + character offset.
//! - `Range::Struct` - `start..end` `Position::Struct` pair.
//! - `HoverRequest::Struct` - inbound request DTO.
//! - `HoverContent::Enum` - `PlainText` / `Markdown` / `Markup` payload.
//! - `HoverResponse::Struct` - outbound response DTO with `contents` list and
//!   optional `Range::Struct`.

/// Hovercontent module.
pub mod HoverContent;

/// Hoverrequest module.
pub mod HoverRequest;

/// Hoverresponse module.
pub mod HoverResponse;

/// Position module.
pub mod Position;

/// Range module.
pub mod Range;
