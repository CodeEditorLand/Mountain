//! Decode percent-encoded characters in URI paths, handling multi-byte UTF-8
//! sequences correctly. Accumulates raw decoded bytes then validates as UTF-8,
//! falling back to lossy conversion for malformed sequences.

pub fn Fn(Input:&str) -> String {
	let mut DecodedBytes:Vec<u8> = Vec::with_capacity(Input.len());

	let Bytes = Input.as_bytes();

	let mut I = 0;

	while I < Bytes.len() {
		if Bytes[I] == b'%' && I + 2 < Bytes.len() {
			let High = HexDigit(Bytes[I + 1]);

			let Low = HexDigit(Bytes[I + 2]);

			if let (Some(H), Some(L)) = (High, Low) {
				DecodedBytes.push(H * 16 + L);

				I += 3;

				continue;
			}
		}

		DecodedBytes.push(Bytes[I]);

		I += 1;
	}

	String::from_utf8(DecodedBytes).unwrap_or_else(|E| String::from_utf8_lossy(E.as_bytes()).into_owned())
}

fn HexDigit(Byte:u8) -> Option<u8> {
	match Byte {
		b'0'..=b'9' => Some(Byte - b'0'),

		b'a'..=b'f' => Some(Byte - b'a' + 10),

		b'A'..=b'F' => Some(Byte - b'A' + 10),

		_ => None,
	}
}
