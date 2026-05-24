//! `WorkspaceContainsGlob::FindMatchingWorkspaceContainsPatterns`



/// Return the subset of `Patterns` for which at least one workspace folder
/// contains a matching file or directory.
pub fn Fn(Folders:&[std::path::PathBuf], Patterns:&[String]) -> Vec<String> {
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

				let Path = Entry.Path();

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
