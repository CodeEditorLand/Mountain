//! `DnsGetZoneInfo` Tauri command - returns the static
//! `land.playform.cloud` zone snapshot (records + DNSSEC flag) used by
//! the diagnostic panel.

use tauri::Struct;

use crate::Binary::Build::{
	DnsCommands::{ZoneInfo::ZoneInfo, ZoneRecord::ZoneRecord},
	Scheme::DnsPort,
};

#[tauri::command]
pub fn Fn(dns_port:State<DnsPort>) -> Result<ZoneInfo, String> {
	if dns_port.0 == 0 {
		return Err("DNS server is not running".to_string());
	}

	let mut records = vec![
		ZoneRecord {
			name:"land.playform.cloud.".to_string(),

			record_type:"SOA".to_string(),

			ttl:3600,

			data:"ns1.land.playform.cloud. admin.land.playform.cloud. 1 3600 600 604800 86400".to_string(),
		},
		ZoneRecord {
			name:"land.playform.cloud.".to_string(),

			record_type:"NS".to_string(),

			ttl:3600,

			data:"ns1.land.playform.cloud.".to_string(),
		},
		ZoneRecord {
			name:"land.playform.cloud.".to_string(),

			record_type:"DNSKEY".to_string(),

			ttl:432000,

			data:"256 3 13 (ECDSA P-256 Zone Signing Key)".to_string(),
		},
		ZoneRecord {
			name:"ns1.land.playform.cloud.".to_string(),

			record_type:"A".to_string(),

			ttl:3600,

			data:"127.0.0.1".to_string(),
		},
		ZoneRecord {
			name:"code.land.playform.cloud.".to_string(),

			record_type:"A".to_string(),

			ttl:3600,

			data:"127.0.0.1".to_string(),
		},
		ZoneRecord {
			name:"api.land.playform.cloud.".to_string(),

			record_type:"A".to_string(),

			ttl:3600,

			data:"127.0.0.1".to_string(),
		},
		ZoneRecord {
			name:"*.land.playform.cloud.".to_string(),

			record_type:"A".to_string(),

			ttl:3600,

			data:"127.0.0.1".to_string(),
		},
	];

	let rrsig_types = vec!["SOA", "NS", "DNSKEY", "A"];

	for rtype in rrsig_types {
		records.push(ZoneRecord {
			name:"land.playform.cloud.".to_string(),
			record_type:"RRSIG".to_string(),
			ttl:432000,
			data:format!("{} 13 2 432000 {} {} {} land.playform.cloud.", rtype, 0, 0, 0),
		});
	}

	let record_count = records.len();

	Ok(ZoneInfo {
		origin:"land.playform.cloud.".to_string(),
		record_count,
		records,
		has_dnssec:true,
	})
}
