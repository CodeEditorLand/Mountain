//! Authoritative-zone snapshot returned by `DnsGetZoneInfo`:
//! origin, record list (one entry per `ZoneRecord::Struct`),
//! and DNSSEC presence flag.

use serde::{Deserialize, Serialize};

use crate::Binary::Build::DnsCommands::ZoneRecord::Struct;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub origin:String,

	pub record_count:usize,

	pub records:Vec<ZoneRecord>,

	pub has_dnssec:bool,
}
