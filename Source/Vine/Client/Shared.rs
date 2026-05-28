//! Canonical state (connection pool, broadcast channel, shutdown flag)
//! lives in `::Vine::Client::Shared`. Mountain's client entry-points
//! all delegate to `::Vine::Client::*` so this module is intentionally
//! empty; Mountain's `Client.rs` declares it to preserve the module path.
