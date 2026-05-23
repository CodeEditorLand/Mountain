//! Fire-and-forget OTLP span exporter. Sends a single
//! `resourceSpans` payload over plain HTTP to the collector at
//! `OTLPEndpoint` (default `127.0.0.1:4318`, configurable via
//! `.env.Land.PostHog`). Stops trying after the first failure
//! (`OTLP_AVAILABLE` flips to `false`) so a missing collector
//! doesn't tax every IPC call. Release builds are compiled out
//! via `cfg!(debug_assertions)`. Honors the `Capture` master
//! telemetry kill switch and the per-pipe `OTLPEnabled` toggle.

use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	sync::{
		OnceLock,
		atomic::{AtomicBool, Ordering},
	},
};

use crate::{Binary::Build::PostHogPlugin::Constants, IPC::DevLog::NowNano};

static OTLP_AVAILABLE:AtomicBool = AtomicBool::new(true);

static OTLP_TRACE_ID:OnceLock<String> = OnceLock::new();

fn GetTraceId() -> &'static str {
	OTLP_TRACE_ID.get_or_init(|| {
		let mut H = DefaultHasher::new();
		std::process::id().hash(&mut H);
		NowNano::Fn().hash(&mut H);
		format!("{:032x}", H.finish() as u128)
	})
}

fn RandU64() -> u64 {
	let mut H = DefaultHasher::new();

	std::thread::current().id().hash(&mut H);

	NowNano::Fn().hash(&mut H);

	H.finish()
}

pub fn Fn(Name:&str, StartNano:u64, EndNano:u64, Attributes:&[(&str, &str)]) {
	if !cfg!(debug_assertions) {
		return;
	}

	if matches!(Constants::TELEMETRY_CAPTURE, "false" | "0" | "off") {
		return;
	}

	if matches!(Constants::OTLP_ENABLED, "false" | "0" | "off") {
		return;
	}

	if !OTLP_AVAILABLE.load(Ordering::Relaxed) {
		return;
	}

	let SpanId = format!("{:016x}", RandU64());

	let TraceId = GetTraceId().to_string();

	let SpanName = Name.to_string();

	let AttributesJson:Vec<String> = Attributes
		.iter()
		.map(|(K, V)| {
			format!(
				r#"{{"key":"{}","value":{{"stringValue":"{}"}}}}"#,
				K,
				V.replace('\\', "\\\\").replace('"', "\\\"")
			)
		})
		.collect();

	let IsError = SpanName.contains("error");

	let StatusCode = if IsError { 2 } else { 1 };

	let Payload = format!(
		concat!(
			r#"{{"resourceSpans":[{{"resource":{{"attributes":["#,
			r#"{{"key":"service.name","value":{{"stringValue":"land-editor-mountain"}}}},"#,
			r#"{{"key":"service.version","value":{{"stringValue":"0.0.1"}}}}"#,
			r#"]}},"scopeSpans":[{{"scope":{{"name":"mountain.ipc","version":"1.0.0"}},"#,
			r#""spans":[{{"traceId":"{}","spanId":"{}","name":"{}","kind":1,"#,
			r#""startTimeUnixNano":"{}","endTimeUnixNano":"{}","#,
			r#""attributes":[{}],"status":{{"code":{}}}}}]}}]}}]}}"#,
		),
		TraceId,
		SpanId,
		SpanName,
		StartNano,
		EndNano,
		AttributesJson.join(","),
		StatusCode,
	);

	// Resolve `OTLPEndpoint` (e.g. `http://127.0.0.1:4318`) → host:port + path.
	// Strip scheme, split on `/` for the path component if any, default to
	// `/v1/traces`.
	let (HostAddress, PathSegment) = ParseEndpoint(Constants::OTLP_ENDPOINT);

	std::thread::spawn(move || {
		use std::{
			io::{Read as IoRead, Write as IoWrite},
			net::TcpStream,
			time::Duration,
		};

		let Ok(SocketAddress) = HostAddress.parse() else {
			OTLP_AVAILABLE.store(false, Ordering::Relaxed);
			return;
		};
		let Ok(mut Stream) = TcpStream::connect_timeout(&SocketAddress, Duration::from_millis(200)) else {
			OTLP_AVAILABLE.store(false, Ordering::Relaxed);
			return;
		};
		let _ = Stream.set_write_timeout(Some(Duration::from_millis(200)));
		let _ = Stream.set_read_timeout(Some(Duration::from_millis(200)));

		let HttpReq = format!(
			"POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: \
			 close\r\n\r\n",
			PathSegment,
			HostAddress,
			Payload.len()
		);
		if Stream.write_all(HttpReq.as_bytes()).is_err() {
			return;
		}
		if Stream.write_all(Payload.as_bytes()).is_err() {
			return;
		}
		let mut Buf = [0u8; 32];
		let _ = Stream.read(&mut Buf);
		if !(Buf.starts_with(b"HTTP/1.1 2") || Buf.starts_with(b"HTTP/1.0 2")) {
			OTLP_AVAILABLE.store(false, Ordering::Relaxed);
		}
	});
}

/// Split `http://host:port/path` into `(host:port, /path)`. Defaults the
/// path to `/v1/traces` when the endpoint has none. Returns owned `String`s
/// so the spawned thread does not borrow the build-time `&'static str`.
fn ParseEndpoint(Endpoint:&str) -> (String, String) {
	let WithoutScheme = Endpoint
		.strip_prefix("http://")
		.or_else(|| Endpoint.strip_prefix("https://"))
		.unwrap_or(Endpoint);

	let (HostPort, Path) = match WithoutScheme.split_once('/') {
		Some((HP, Rest)) => (HP.to_string(), format!("/{}", Rest.trim_start_matches('/'))),

		None => (WithoutScheme.to_string(), "/v1/traces".to_string()),
	};

	let PathFinal = if Path == "/" { "/v1/traces".to_string() } else { Path };

	(HostPort, PathFinal)
}
