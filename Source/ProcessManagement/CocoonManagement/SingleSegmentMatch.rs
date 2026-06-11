//! Star-glob match on a single path segment for `workspaceContains:`
//! patterns. `?` is unsupported; unsupported glob chars fall through to
//! literal equality.

pub(crate) fn Fn(Pattern:&str, Segment:&str) -> bool {
	if Pattern == "*" {
		return true;
	}

	if !Pattern.contains('*') && !Pattern.contains('?') {
		return Pattern == Segment;
	}

	// Minimal star-glob on a single segment: split by '*' and check each
	// fragment appears in order. Doesn't support `?` (rare in
	// workspaceContains patterns); unsupported glob chars fall through to
	// literal equality.
	let Fragments:Vec<&str> = Pattern.split('*').collect();

	let mut Cursor = 0usize;

	for (Index, Fragment) in Fragments.iter().enumerate() {
		if Fragment.is_empty() {
			continue;
		}

		if Index == 0 {
			if !Segment[Cursor..].starts_with(Fragment) {
				return false;
			}

			Cursor += Fragment.len();

			continue;
		}

		match Segment[Cursor..].find(Fragment) {
			Some(Offset) => Cursor += Offset + Fragment.len(),

			None => return false,
		}
	}

	if let Some(Last) = Fragments.last()
		&& !Last.is_empty()
	{
		return Segment.ends_with(Last);
	}

	true
}
