//! `WorkspaceContainsGlob::SegmentMatch`



/// Recursive segment-by-segment glob match. `**` consumes zero or more
/// path segments; `*` matches exactly one segment via `SingleSegmentMatch`.
pub fn Fn(Pattern:&[&str], Entry:&[&str]) -> bool {
	if Pattern.is_empty() {
		return Entry.is_empty();
	}

	let Head = Pattern[0];

	if Head == "**" {
		for Consumed in 0..=Entry.len() {
			if SegmentMatch(&Pattern[1..], &Entry[Consumed..]) {
				return true;
			}
		}

		return false;
	}

	if Entry.is_empty() {
		return false;
	}

	if SingleSegmentMatch(Head, Entry[0]) {
		return SegmentMatch(&Pattern[1..], &Entry[1..]);
	}

	false
}
