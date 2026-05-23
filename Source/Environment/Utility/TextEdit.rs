#![allow(unused_variables, dead_code, unused_imports)]

//! Pure text-editing utilities shared across workspace and document providers.
//!
//! These are side-effect-free helper functions that compute line offsets and
//! translate `(line, character)` positions to byte offsets, matching VS Code's
//! UTF-16 code unit counting convention for `Range`/`Position` values.

/// Pre-compute the byte offset of the start of every line in `Source`.
/// The returned vec always has at least one entry (`[0]`).
pub(crate) fn ComputeLineOffsets(Source:&str) -> Vec<usize> {
	let mut Offsets = Vec::with_capacity(Source.len() / 40 + 1);

	Offsets.push(0);

	for (Index, Byte) in Source.bytes().enumerate() {
		if Byte == b'\n' {
			Offsets.push(Index + 1);
		}
	}

	Offsets
}

/// Resolve `(line, character)` to an absolute byte offset in `Source`.
/// `character` is counted in **UTF-16 code units** to match VS Code's
/// `Range`/`Position` semantics. Falls back to EOF when line/character
/// exceeds the source length.
pub(crate) fn LinePosToOffset(LineOffsets:&[usize], Source:&str, Line:usize, Character:usize) -> usize {
	if Line >= LineOffsets.len() {
		return Source.len();
	}

	let LineStart = LineOffsets[Line];

	let LineEnd = if Line + 1 < LineOffsets.len() {
		LineOffsets[Line + 1].saturating_sub(1)
	} else {
		Source.len()
	};

	let LineText = &Source[LineStart..LineEnd.min(Source.len())];

	let mut Utf16Count:usize = 0;

	for (ByteOffset, Char) in LineText.char_indices() {
		if Utf16Count >= Character {
			return LineStart + ByteOffset;
		}

		Utf16Count += Char.len_utf16();
	}

	LineStart + LineText.len()
}

/// Minimal percent-decoder for `file://` URI paths. Self-contained to avoid
/// an extra crate dependency; handles `%XX` sequences only.
pub(crate) fn percent_decode(Input:&str) -> String {
	let mut Out = String::with_capacity(Input.len());

	let mut Bytes = Input.as_bytes().iter().peekable();

	while let Some(&Byte) = Bytes.next() {
		if Byte == b'%' {
			let H = Bytes.next().copied();

			let L = Bytes.next().copied();

			if let (Some(H), Some(L)) = (H, L) {
				if let (Some(Hi), Some(Lo)) = (hex_digit(H), hex_digit(L)) {
					Out.push((Hi * 16 + Lo) as char);

					continue;
				}

				Out.push('%');

				Out.push(H as char);

				Out.push(L as char);

				continue;
			}

			Out.push('%');
		} else {
			Out.push(Byte as char);
		}
	}

	Out
}

fn hex_digit(Byte:u8) -> Option<u8> {
	match Byte {
		b'0'..=b'9' => Some(Byte - b'0'),

		b'a'..=b'f' => Some(Byte - b'a' + 10),

		b'A'..=b'F' => Some(Byte - b'A' + 10),

		_ => None,
	}
}
