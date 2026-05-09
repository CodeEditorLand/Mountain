#![allow(non_snake_case)]

//! Authoritative-zone snapshot returned by `dns_get_zone_info`:
//! origin, record list (one entry per `ZoneRecord::Struct`),
//! and DNSSEC presence flag.

use serde::{Deserialize, Serialize};

use crate::Binary::Build::DnsCommands::ZoneRecord::ZoneRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneInfo {

	pub origin:String,

	pub record_count:usize,

	pub records:Vec<ZoneRecord>,

	pub has_dnssec:bool,
}
