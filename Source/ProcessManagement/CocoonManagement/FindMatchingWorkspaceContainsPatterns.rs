//! Bounded workspace walk that returns the subset of `workspaceContains:`
//! patterns matched by at least one workspace folder.

/// Return the subset of `Patterns` for which at least one workspace folder
/// contains a matching file or directory. Patterns are interpreted the same
/// way VS Code does for `workspaceContains:<pattern>` activation events:
///
/// - A bare filename (no slash, no wildcards) matches an entry with that name
///   at the workspace root (e.g. `package.json`).
/// - A path with slashes but no wildcards matches a direct descendant relative
///   to the root (e.g. `.vscode/launch.json`).
/// - A glob with `**/` prefix matches any descendant up to a bounded depth.
/// - Any other wildcard form is matched via a simple segment-by-segment walk
///   honouring `*` (single segment) and `**` (any number of segments).
///
/// Matching is bounded to depth 3 and 4096 total directory entries per
/// workspace root to keep the cost sub-100 ms on large monorepos. Anything
/// deeper is rare for activation-event triggers; the trade-off is
/// documented in VS Code's own `ExtensionService.scanExtensions`.
pub(crate) fn Fn(Folders:&[std::path::PathBuf], Patterns:&[String]) -> Vec<String> {
	use std::collections::HashSet;

	const MAX_DEPTH:usize = 3;

	const MAX_ENTRIES_PER_ROOT:usize = 4096;

	let mut Matched:HashSet<String> = HashSet::new();

	for Folder in Folders {
		if !Folder.is_dir() {
			continue;
		}

		// Collect up to MAX_ENTRIES_PER_ROOT paths relative to the folder.
		let mut Entries:Vec<String> = Vec::new();

		let mut Stack:Vec<(std::path::PathBuf, usize)> = vec![(Folder.clone(), 0)];

		while let Some((Current, Depth)) = Stack.pop() {
			if Entries.len() >= MAX_ENTRIES_PER_ROOT {
				break;
			}

			let ReadDirResult = std::fs::read_dir(&Current);

			let ReadDir = match ReadDirResult {
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

			if super::PatternMatchesAnyEntry::Fn(Pattern, &Entries) {
				Matched.insert(Pattern.clone());
			}
		}
	}

	Matched.into_iter().collect()
}
