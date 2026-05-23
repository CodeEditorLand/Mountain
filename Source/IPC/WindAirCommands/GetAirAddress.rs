
//! Resolve the Air daemon's gRPC address. Currently hard-coded
//! to `DEFAULT_AIR_SERVER_ADDRESS`; future revisions will read
//! a runtime config slot.

use crate::Air::AirClient::DEFAULT_AIR_SERVER_ADDRESS;

pub(super) fn Fn() -> Result<String, String> { Ok(DEFAULT_AIR_SERVER_ADDRESS.to_string()) }
