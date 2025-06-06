
// Defines a helper function for analyzing text content.

#![allow(non_snake_case, non_camel_case_types)]

/// Analyzes a string to determine its line ending style (`\n` or `\r\n`)
/// and splits the string into a vector of lines based on that ending.
pub fn AnalyzeTextLinesAndEol(TextContent:&str) -> (Vec<String>, String) {
	let DetectedEol = if TextContent.contains("\r\n") { "\r\n" } else { "\n" };
	let LineList = TextContent.split(DetectedEol).map(String::from).collect();
	(LineList, DetectedEol.to_string())
}
