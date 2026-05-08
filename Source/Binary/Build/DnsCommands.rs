#![allow(non_snake_case)]

//! # DNS commands
//!
//! Tauri commands that surface the Mist-managed DNS server
//! (Hickory) to the webview - server state, zone snapshot,
//! forward allowlist, manual resolution. The DTOs and command
//! handlers live in sibling files; the wire-bound names match
//! the file names so the `invoke_handler!` registration in
//! `Binary/Main/Entry.rs` is a 1:1 mapping.

pub mod DnsHealthStatus;

pub mod DnsResolutionResult;

pub mod DnsServerInfo;

pub mod ForwardAllowList;

pub mod StartupTime;

pub mod ZoneInfo;

pub mod ZoneRecord;

pub mod dns_get_forward_allowlist;

pub mod dns_get_health_status;

pub mod dns_get_server_info;

pub mod dns_get_zone_info;

pub mod dns_health_check;

pub mod dns_resolve;

pub mod dns_test_resolution;

#[cfg(test)]
mod tests {

	use super::{
		DnsHealthStatus::DnsHealthStatus,
		DnsResolutionResult::DnsResolutionResult,
		DnsServerInfo::DnsServerInfo,
		ForwardAllowList::ForwardAllowList,
		ZoneRecord::ZoneRecord,
	};

	#[test]
	fn DnsServerInfoSerialization() {
		let info = DnsServerInfo { port:5380, is_running:true, startup_time:"2024-01-01T00:00:00Z".to_string() };

		let json = serde_json::to_string(&info).unwrap();

		let deserialized:DnsServerInfo = serde_json::from_str(&json).unwrap();

		assert_eq!(deserialized.port, 5380);

		assert_eq!(deserialized.is_running, true);

		assert_eq!(deserialized.startup_time, "2024-01-01T00:00:00Z");
	}

	#[test]
	fn ZoneRecordSerialization() {
		let record = ZoneRecord {
			name:"code.editor.land.".to_string(),

			record_type:"A".to_string(),

			ttl:3600,

			data:"127.0.0.1".to_string(),
		};

		let json = serde_json::to_string(&record).unwrap();

		let deserialized:ZoneRecord = serde_json::from_str(&json).unwrap();

		assert_eq!(deserialized.name, "code.editor.land.");

		assert_eq!(deserialized.record_type, "A");

		assert_eq!(deserialized.ttl, 3600);

		assert_eq!(deserialized.data, "127.0.0.1");
	}

	#[test]
	fn ForwardAllowListSerialization() {
		let allowlist = ForwardAllowList { domains:vec!["update.editor.land.".to_string()] };

		let json = serde_json::to_string(&allowlist).unwrap();

		let deserialized:ForwardAllowList = serde_json::from_str(&json).unwrap();

		assert_eq!(deserialized.domains.len(), 1);

		assert_eq!(deserialized.domains[0], "update.editor.land.");
	}

	#[test]
	fn DnsHealthStatusSerialization() {
		let health = DnsHealthStatus {
			server_status:"running".to_string(),

			zone_status:"active".to_string(),

			forward_status:"active".to_string(),

			last_error:None,
		};

		let json = serde_json::to_string(&health).unwrap();

		let deserialized:DnsHealthStatus = serde_json::from_str(&json).unwrap();

		assert_eq!(deserialized.server_status, "running");

		assert_eq!(deserialized.zone_status, "active");

		assert_eq!(deserialized.forward_status, "active");

		assert!(deserialized.last_error.is_none());
	}

	#[test]
	fn DnsResolutionResultSerialization() {
		let result = DnsResolutionResult {
			domain:"code.editor.land.".to_string(),

			record_type:"A".to_string(),

			addresses:vec!["127.0.0.1".to_string()],

			ttl:3600,

			succeeded:true,

			error:None,
		};

		let json = serde_json::to_string(&result).unwrap();

		let deserialized:DnsResolutionResult = serde_json::from_str(&json).unwrap();

		assert_eq!(deserialized.domain, "code.editor.land.");

		assert_eq!(deserialized.record_type, "A");

		assert_eq!(deserialized.addresses.len(), 1);

		assert_eq!(deserialized.addresses[0], "127.0.0.1");

		assert_eq!(deserialized.ttl, 3600);

		assert!(deserialized.succeeded);

		assert!(deserialized.error.is_none());
	}
}
