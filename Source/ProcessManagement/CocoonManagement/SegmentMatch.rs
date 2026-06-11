//! Segment-by-segment glob walk for `workspaceContains:` patterns,
//! honouring `*` (one segment) and `**` (zero or more segments).

pub(crate) fn Fn(Pattern:&[&str], Entry:&[&str]) -> bool {
	if Pattern.is_empty() {
		return Entry.is_empty();
	}

	let Head = Pattern[0];

	if Head == "**" {
		// `**` matches zero or more segments. Try consuming 0..=entry.len().
		for Consumed in 0..=Entry.len() {
			if Fn(&Pattern[1..], &Entry[Consumed..]) {
				return true;
			}
		}

		return false;
	}

	if Entry.is_empty() {
		return false;
	}

	if super::SingleSegmentMatch::Fn(Head, Entry[0]) {
		return Fn(&Pattern[1..], &Entry[1..]);
	}

	false
}
