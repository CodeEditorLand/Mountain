//! # URI Parsing Utilities
//!
//! Functions for parsing and converting URI/URL representations.

use url::Url;
use CommonLibrary::Error::CommonError::CommonError;

/// Helper to get a `Url` from a `serde_json::Value` which is expected to be a
/// `UriComponents` DTO from VS Code, a plain URI string, or a UriComponents
/// object without the `external` convenience field.
///
/// Cocoon's wire shapes vary by call site:
///   - `Diagnostic.Set` and a few others send the URI as a plain string (the
///     canonical form returned by `Uri.toString()`).
///   - `MainThread*` boundaries send the `{scheme, authority, path, query,
///     fragment, external}` object that vs/base/common/uri.ts emits via
///     `URI.toJSON()`.
///   - Some legacy paths send `{scheme, path}` with no `external`.
///
/// All three are valid; Mountain accepts whichever arrives. Without this the
/// Diagnostic.Set call from `vscode.languages.createDiagnosticCollection().set`
/// trips the breaker after 5 publishes and silences every linter / compiler
/// across all language extensions.
pub fn GetURLFromURIComponentsDTO(URIDTO:&serde_json::Value) -> Result<Url, CommonError> {
	// 1. Plain string: parse directly.
	if let Some(URIString) = URIDTO.as_str() {
		return Url::parse(URIString).map_err(|Error| {
			CommonError::InvalidArgument {
				ArgumentName:"URIDTO".to_string(),
				Reason:format!("Failed to parse URI string '{}': {}", URIString, Error),
			}
		});
	}

	// 2. Object with `external` field (VS Code's UriComponents).
	if let Some(URIString) = URIDTO.get("external").and_then(serde_json::Value::as_str) {
		return Url::parse(URIString).map_err(|Error| {
			CommonError::InvalidArgument {
				ArgumentName:"URIDTO.external".to_string(),
				Reason:format!("Failed to parse URI string '{}': {}", URIString, Error),
			}
		});
	}

	// 3. Object with scheme/authority/path - reconstruct the URI string.
	let Scheme = URIDTO.get("scheme").and_then(serde_json::Value::as_str);

	let Path = URIDTO.get("path").and_then(serde_json::Value::as_str);

	if let (Some(Scheme), Some(Path)) = (Scheme, Path) {
		let Authority = URIDTO.get("authority").and_then(serde_json::Value::as_str).unwrap_or("");

		let Query = URIDTO.get("query").and_then(serde_json::Value::as_str).unwrap_or("");

		let Fragment = URIDTO.get("fragment").and_then(serde_json::Value::as_str).unwrap_or("");

		let mut Reconstructed = format!("{}://{}{}", Scheme, Authority, Path);

		if !Query.is_empty() {
			Reconstructed.push('?');

			Reconstructed.push_str(Query);
		}

		if !Fragment.is_empty() {
			Reconstructed.push('#');

			Reconstructed.push_str(Fragment);
		}

		return Url::parse(&Reconstructed).map_err(|Error| {
			CommonError::InvalidArgument {
				ArgumentName:"URIDTO".to_string(),
				Reason:format!("Failed to parse reconstructed URI '{}': {}", Reconstructed, Error),
			}
		});
	}

	Err(CommonError::InvalidArgument {
		ArgumentName:"URIDTO".to_string(),
		Reason:"Expected a URI string, an object with 'external', or an object with 'scheme' + 'path'".to_string(),
	})
}
