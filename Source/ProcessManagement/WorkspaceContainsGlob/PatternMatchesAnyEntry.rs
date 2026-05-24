//! `WorkspaceContainsGlob::PatternMatchesAnyEntry`



/// Check whether `Pattern` matches any entry in `Entries`.
/// Supports literal paths, `*` (one segment), and `**` (any segments).
/// Case-sensitive per the VS Code spec.
pub fn Fn(Pattern:&str, Entries:&[String]) -> bool {
	let HasWildcard = Pattern.contains('*') || Pattern.contains('?');

	if !HasWildcard {
		return Entries.iter().any(|E| E == Pattern);
	}

	let PatternSegments:Vec<&str> = Pattern.split('/').collect();

	Entries
		.iter()
		.any(|E| SegmentMatch(&PatternSegments, &E.split('/').collect::<Vec<_>>()))
}
