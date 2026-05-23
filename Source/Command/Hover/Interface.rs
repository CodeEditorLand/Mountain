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

pub mod HoverContent;

pub mod HoverRequest;

pub mod HoverResponse;

pub mod Position;

pub mod Range;
