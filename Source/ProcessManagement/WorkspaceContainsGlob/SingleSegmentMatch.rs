//! `WorkspaceContainsGlob::SingleSegmentMatch`



/// Match a single path segment against a pattern that may contain `*`.
/// `?` is not supported (rare in workspaceContains patterns) and falls
/// through to literal equality.
pub fn Fn(Pattern:&str, Segment:&str) -> bool {
	if Pattern == "*" {
		return true;
	}

	if !Pattern.contains('*') && !Pattern.contains('?') {
		return Pattern == Segment;
	}

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
