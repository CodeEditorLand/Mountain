//! MIME type detection from a file extension.

/// MIME type detection from file extension
pub(crate) fn Fn(Path:&str) -> &'static str {
	if Path.ends_with(".js") || Path.ends_with(".mjs") {
		"application/javascript"
	} else if Path.ends_with(".css") {
		"text/css"
	} else if Path.ends_with(".html") || Path.ends_with(".htm") {
		"text/html"
	} else if Path.ends_with(".json") {
		"application/json"
	} else if Path.ends_with(".svg") {
		"image/svg+xml"
	} else if Path.ends_with(".png") {
		"image/png"
	} else if Path.ends_with(".jpg") || Path.ends_with(".jpeg") {
		"image/jpeg"
	} else if Path.ends_with(".gif") {
		"image/gif"
	} else if Path.ends_with(".woff") {
		"font/woff"
	} else if Path.ends_with(".woff2") {
		"font/woff2"
	} else if Path.ends_with(".ttf") {
		"font/ttf"
	} else if Path.ends_with(".wasm") {
		"application/wasm"
	} else if Path.ends_with(".map") {
		"application/json"
	} else if Path.ends_with(".txt") || Path.ends_with(".md") {
		"text/plain"
	} else if Path.ends_with(".xml") {
		"application/xml"
	} else {
		"application/octet-stream"
	}
}
