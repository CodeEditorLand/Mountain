//! # AnalyzeTextLinesAndEOL Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Analyzes text content to determine its line endings and splits it into a
//! vector of lines for document state management.
//!
//! ## ARCHITECTURAL ROLE
//! AnalyzeTextLinesAndEOL is part of the **Internal::TextProcessing** module,
//! providing text analysis utilities.
//!
//! ## KEY COMPONENTS
//! - AnalyzeTextLinesAndEOL: Function to analyze text lines and EOL
//!
//! ## ERROR HANDLING
//! - Detects both CRLF and LF line endings
//! - Returns safe defaults for empty text
//!
//! ## LOGGING
//! Operations are logged at appropriate levels (debug).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient line splitting
//! - EOL detection
//!
//! ## TODO
//! - [ ] Add encoding detection
//! - [ ] Implement line ending normalization
//! - [ ] Add performance metrics


/// Analyzes text content to determine its line endings and splits it into a
/// vector of lines.
///
/// # Arguments
/// * `TextContent` - The text content to analyze
///
/// # Returns
/// Tuple containing (`Vec<String>` of lines, String of detected EOL)
///
/// # Behavior
/// - Detects CRLF ("\r\n") or LF ("\n") line endings
/// - Splits text into lines vector using detected EOL
/// - Returns LF as default if text doesn't contain CRLF
pub fn AnalyzeTextLinesAndEOL(TextContent:&str) -> (Vec<String>, String) {
	let detected_eol = if TextContent.contains("\r\n") {
		dev_log!("model", "[AnalyzeTextLinesAndEOL] Detected CRLF line endings");
		"\r\n"
	} else {
		dev_log!("model", "[AnalyzeTextLinesAndEOL] Detected LF line endings");
		"\n"
	};

	let lines:Vec<String> = TextContent.split(detected_eol).map(String::from).collect();

	dev_log!("model", 
		"[AnalyzeTextLinesAndEOL] Analyzed {} lines with EOL: {:?}",
		lines.len(),
		detected_eol
	);

	(lines, detected_eol.to_string())
}
use crate::dev_log;
