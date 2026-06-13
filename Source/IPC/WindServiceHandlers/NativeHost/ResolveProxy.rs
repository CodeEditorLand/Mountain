//! Wire method: `nativeHost:resolveProxy`.
//!
//! Resolves proxy settings for a given URL. Returns `DIRECT` when no
//! proxy env var is set, or the var's value in PAC format when one is
//! configured. VS Code uses this before every authenticated HTTP request.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_str;

pub fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Url = arg_str(&Arguments, 0);

	let Scheme = if Url.starts_with("https") { "HTTPS" } else { "HTTP" };

	let ProxyEnv = std::env::var(format!("{}_PROXY", Scheme))
		.or_else(|_| std::env::var(format!("{}_proxy", Scheme.to_lowercase())))
		.or_else(|_| std::env::var("ALL_PROXY"))
		.or_else(|_| std::env::var("all_proxy"));

	match ProxyEnv {
		Ok(P) if !P.is_empty() => {
			let Lower = P.to_lowercase();

			let (Keyword, Host) = if Lower.starts_with("socks") {
				let H = P
					.trim_start_matches("socks5://")
					.trim_start_matches("socks4://")
					.trim_start_matches("socks://");

				("SOCKS", H)
			} else {
				let H = P.trim_start_matches("http://").trim_start_matches("https://");

				("PROXY", H)
			};

			Ok(json!(format!("{} {}", Keyword, Host)))
		},

		_ => Ok(json!("DIRECT")),
	}
}
