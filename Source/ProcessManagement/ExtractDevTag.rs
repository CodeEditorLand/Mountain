#![allow(non_snake_case)]
//! Cocoon stdout line inspector: detects the `[DEV:<TAG>] <body>` prefix
//! `Cocoon/Source/Services/DevLog.ts::CocoonDevLog` writes and returns
//! the lowercased tag for dispatch into Mountain's per-tag `dev_log!`
//! sinks. Returns `None` for bare stdout so the caller falls back to
//! the catch-all `cocoon` tag.

pub fn ExtractDevTag(Line:&str) -> Option<String> {
	let Stripped = Line.strip_prefix("[DEV:")?;
	let (TagUpper, _Rest) = Stripped.split_once(']')?;
	if TagUpper.is_empty() {
		return None;
	}
	// Reject anything that isn't a simple tag ident - prevents stray
	// `[DEV: something with space]` headers from being treated as tags.
	if !TagUpper.chars().all(|C| C.is_ascii_uppercase() || C == '-' || C == '_') {
		return None;
	}
	Some(TagUpper.to_ascii_lowercase())
}

#[cfg(test)]
mod Tests {
	use super::ExtractDevTag;

	#[test]
	fn StripsKnownTag() {
		assert_eq!(
			ExtractDevTag("[DEV:BOOTSTRAP-STAGE] [Bootstrap] stage=Environment event=start"),
			Some("bootstrap-stage".to_string())
		);
	}

	#[test]
	fn RejectsPlainText() { assert_eq!(ExtractDevTag("plain stdout line"), None); }

	#[test]
	fn RejectsMalformed() {
		assert_eq!(ExtractDevTag("[DEV: BOOT] x"), None);
		assert_eq!(ExtractDevTag("[DEV:]"), None);
		assert_eq!(ExtractDevTag("[DEV:BOOT"), None);
	}
}
