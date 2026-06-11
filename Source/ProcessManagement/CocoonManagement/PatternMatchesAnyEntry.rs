//! Very small glob-matcher scoped to VS Code `workspaceContains:` syntax.
//! Supports literal paths, `*` (one path segment), and `**` (zero or more
//! segments). Case-sensitive per the VS Code spec.

pub(crate) fn Fn(Pattern:&str, Entries:&[String]) -> bool {
	let HasWildcard = Pattern.contains('*') || Pattern.contains('?');

	if !HasWildcard {
		return Entries.iter().any(|E| E == Pattern);
	}

	let PatternSegments:Vec<&str> = Pattern.split('/').collect();

	Entries
		.iter()
		.any(|E| super::SegmentMatch::Fn(&PatternSegments, &E.split('/').collect::<Vec<_>>()))
}
