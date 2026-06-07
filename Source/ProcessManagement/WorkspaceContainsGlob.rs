//! Workspace-contains glob matcher for VS Code `workspaceContains:<pattern>`
//! activation events.
//!
//! Matching semantics mirror VS Code's `ExtensionService.scanExtensions`:
//! - Bare filename → exact match at workspace root
//! - Path with slashes → direct descendant match
//! - `**/pattern` → any descendant up to depth 3
//! - Single `*` → one path segment wildcard
//!
//! Bounded to depth 3 and 4096 entries per root so activation checks stay
//! sub-100 ms on large monorepos.

/// Return the subset of `Patterns` for which at least one workspace folder
/// contains a matching file or directory.
pub fn FindMatchingWorkspaceContainsPatterns(Folders:&[std::path::PathBuf], Patterns:&[String]) -> Vec<String> {

	use std::collections::HashSet;

	const MAX_DEPTH:usize = 3;

	const MAX_ENTRIES_PER_ROOT:usize = 4096;

	let mut Matched:HashSet<String> = HashSet::new();

	for Folder in Folders {
		if !Folder.is_dir() {
			continue;
		}

		let mut Entries:Vec<String> = Vec::new();

		let mut Stack:Vec<(std::path::PathBuf, usize)> = vec![(Folder.clone(), 0)];

		while let Some((Current, Depth)) = Stack.pop() {
			if Entries.len() >= MAX_ENTRIES_PER_ROOT {
				break;
			}

			let ReadDir = match std::fs::read_dir(&Current) {
				Ok(R) => R,

				Err(_) => continue,
			};

			for Entry in ReadDir.flatten() {
				if Entries.len() >= MAX_ENTRIES_PER_ROOT {
					break;
				}

				let Path = Entry.path();

				let Relative = match Path.strip_prefix(Folder) {
					Ok(R) => R.to_string_lossy().replace('\\', "/"),

					Err(_) => continue,
				};

				let IsDir = Entry.file_type().map(|T| T.is_dir()).unwrap_or(false);

				Entries.push(Relative.clone());

				if IsDir && Depth + 1 < MAX_DEPTH {
					Stack.push((Path, Depth + 1));
				}
			}
		}

		for Pattern in Patterns {
			if Matched.contains(Pattern) {
				continue;
			}

			if PatternMatchesAnyEntry(Pattern, &Entries) {
				Matched.insert(Pattern.clone());
			}
		}
	}

	Matched.into_iter().collect()
}

/// Check whether `Pattern` matches any entry in `Entries`.
/// Supports literal paths, `*` (one segment), and `**` (any segments).
/// Case-sensitive per the VS Code spec.
pub fn PatternMatchesAnyEntry(Pattern:&str, Entries:&[String]) -> bool {

	let HasWildcard = Pattern.contains('*') || Pattern.contains('?');

	if !HasWildcard {
		return Entries.iter().any(|E| E == Pattern);
	}

	let PatternSegments:Vec<&str> = Pattern.split('/').collect();

	Entries
		.iter()
		.any(|E| SegmentMatch(&PatternSegments, &E.split('/').collect::<Vec<_>>()))
}

/// Recursive segment-by-segment glob match. `**` consumes zero or more
/// path segments; `*` matches exactly one segment via `SingleSegmentMatch`.
pub fn SegmentMatch(Pattern:&[&str], Entry:&[&str]) -> bool {

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

/// Match a single path segment against a pattern that may contain `*`.
/// `?` is not supported (rare in workspaceContains patterns) and falls
/// through to literal equality.
pub fn SingleSegmentMatch(Pattern:&str, Segment:&str) -> bool {

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
