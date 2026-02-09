//! # URI Parsing Utilities
//!
//! Functions for parsing and converting URI/URL representations.

use url::Url;
use CommonLibrary::Error::CommonError::CommonError;

/// Helper to get a `Url` from a `serde_json::Value` which is expected to be a
/// `UriComponents` DTO from VS Code.
pub fn GetURLFromURIComponentsDTO(URIDTO:&serde_json::Value) -> Result<Url, CommonError> {
	// VS Code's UriComponents DTO often serializes to an object with a path,
	// scheme, etc., but also includes a pre-formatted 'external' string version.
	let URIString = URIDTO.get("external").and_then(serde_json::Value::as_str).ok_or_else(|| {
		CommonError::InvalidArgument {
			ArgumentName:"URIDTO".to_string(),
			Reason:"Missing 'external' string field in UriComponents DTO".to_string(),
		}
	})?;

	Url::parse(URIString).map_err(|Error| {
		CommonError::InvalidArgument {
			ArgumentName:"URIDTO.external".to_string(),
			Reason:format!("Failed to parse URI string '{}': {}", URIString, Error),
		}
	})
}
