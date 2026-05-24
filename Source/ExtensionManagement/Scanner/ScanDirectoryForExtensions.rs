//! `Scanner::ScanDirectoryForExtensions`

use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory, ReadFile::ReadFile},
};
use serde_json::{Map, Value};
use tauri::Manager;
use crate::{
	ApplicationState::{
		DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
		Struct::ApplicationState::ApplicationState,
	},
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

const EXTENSION_SCAN_DENY_LIST:&[&str] = &["types", "out", "node_modules", "test", ".vscode-test", ".git"];
const TEST_ONLY_EXTENSIONS:&[&str] = &[
	"vscode-api-tests",
	"vscode-test-resolver",
	"vscode-colorize-tests",
	"vscode-colorize-perf-tests",
	"vscode-notebook-tests",
];

/// Scans a single directory for valid extensions.
///
/// This function iterates through a given directory, looking for subdirectories
/// that contain a `package.json` file. It then attempts to parse this file
/// into an `ExtensionDescriptionStateDTO`.
pub async fn Fn(
	ApplicationHandle:tauri::AppHandle,

	DirectoryPath:PathBuf,
) -> Result<Vec<ExtensionDescriptionStateDTO>, CommonError> {
	// Decide up-front whether this scan path contributes built-ins or user
	// extensions. Built-ins are ones shipped inside the Mountain/Sky/VS Code
	// bundle; the `~/.fiddee/extensions` root is user-space.
	let IsUserPath = IsUserExtensionScanPath(&DirectoryPath);

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let mut FoundExtensions = Vec::new();

	// Distinguish "directory does not exist" (first-run, no user extensions
	// installed yet - perfectly normal) from a real I/O failure. Only the
	// latter deserves a `warn:` prefix; the former is debug-level noise.
	match DirectoryPath.try_exists() {
		Ok(false) => {
			dev_log!(
				"extensions",
				"[ExtensionScanner] Extension path '{}' does not exist, skipping (no extensions installed here)",
				DirectoryPath.display()
			);

			return Ok(Vec::new());
		},

		Err(error) => {
			dev_log!(
				"extensions",
				"[ExtensionScanner] Could not stat extension path '{}': {} - skipping",
				DirectoryPath.display(),
				error
			);

			return Ok(Vec::new());
		},

		Ok(true) => {},
	}

	let TopLevelEntries = match RunTime.Run(ReadDirectory(DirectoryPath.clone())).await {
		Ok(entries) => entries,

		Err(error) => {
			dev_log!(
				"extensions",
				"warn: [ExtensionScanner] Could not read extension directory '{}': {}. Skipping.",
				DirectoryPath.display(),
				error
			);

			return Ok(Vec::new());
		},
	};

	dev_log!(
		"extensions",
		"[ExtensionScanner] Directory '{}' contains {} top-level entries",
		DirectoryPath.display(),
		TopLevelEntries.len()
	);

	let mut parse_failures = 0usize;

	let mut missing_package_json = 0usize;

	let mut denied_directory_count = 0usize;

	let mut test_extension_skips = 0usize;

	let AllowTestExtensions = IncludeTestExtensions();

	for (EntryName, FileType) in TopLevelEntries {
		if FileType == FileTypeDTO::Directory {
			// BATCH-18: skip scanner traversal into directories that are
			// build output / shared deps, not extensions.
			if IsDeniedDirectory(&EntryName) {
				denied_directory_count += 1;

				continue;
			}

			if !AllowTestExtensions && IsTestOnlyExtension(&EntryName) {
				test_extension_skips += 1;

				continue;
			}

			let PotentialExtensionPath = DirectoryPath.join(EntryName);

			let PackageJsonPath = PotentialExtensionPath.join("package.json");

			// Per-candidate-directory probe, fires for every top-level
			// entry the scanner inspects (203 lines per session). The
			// accepted / rejected disposition is already covered by the
			// `ext-scan` tag below.
			dev_log!(
				"ext-scan-verbose",
				"[ExtensionScanner] Checking for package.json in: {}",
				PotentialExtensionPath.display()
			);

			match RunTime.Run(ReadFile(PackageJsonPath.clone())).await {
				Ok(PackageJsonContent) => {
					// Parse to a dynamic JSON value first so we can resolve
					// VS Code NLS placeholders (`%key%` strings referencing
					// `package.nls.json` entries) across every typed field.
					// Without this the UI renders literal `%command.clone%`,
					// `%displayName%`, etc. in the Command Palette and menus.
					let mut ManifestValue:Value = match serde_json::from_slice::<Value>(&PackageJsonContent) {
						Ok(v) => v,

						Err(error) => {
							parse_failures += 1;

							dev_log!(
								"extensions",
								"warn: [ExtensionScanner] Failed to parse package.json at '{}': {}",
								PotentialExtensionPath.display(),
								error
							);

							continue;
						},
					};

					// BATCH-18: only report "no bundle" when the manifest
					// actually contains `%placeholder%` strings that need
					// substitution. Many shipped extensions (js-debug-companion,
					// js-profile-table) publish English-only manifests with no
					// placeholders - surfacing a warning there is misleading
					// because the UI renders correctly with the raw fields.
					let ManifestUsesPlaceholders = ManifestContainsNLSPlaceholders(&ManifestValue);

					if let Some(NLSMap) =
						LoadNLSBundle(&RunTime, &PotentialExtensionPath, ManifestUsesPlaceholders).await
					{
						let mut Replaced = 0u32;

						let mut Unresolved = 0u32;

						ResolveNLSPlaceholdersInner(&mut ManifestValue, &NLSMap, &mut Replaced, &mut Unresolved);

						dev_log!(
							"nls",
							"[LandFix:NLS] {} → {} replaced, {} unresolved placeholders",
							PotentialExtensionPath.display(),
							Replaced,
							Unresolved
						);
					}

					match serde_json::from_value::<ExtensionDescriptionStateDTO>(ManifestValue) {
						Ok(mut Description) => {
							// Augment the description with its location on disk.
							Description.ExtensionLocation =
								serde_json::to_value(url::Url::from_directory_path(&PotentialExtensionPath).unwrap())
									.unwrap_or(Value::Null);

							// Construct identifier from publisher.name if not set
							if Description.Identifier == Value::Null
								|| Description.Identifier == Value::Object(Default::default())
							{
								let Id = if Description.Publisher.is_empty() {
									Description.Name.clone()
								} else {
									format!("{}.{}", Description.Publisher, Description.Name)
								};

								Description.Identifier = serde_json::json!({ "value": Id });
							}

							// Classify the extension by the scan path it came from.
							// Built-in extensions ship in the Mountain/Sky/VS Code
							// bundle; user extensions live under
							// `~/.fiddee/extensions` (written by
							// `VsixInstaller::InstallVsix`). Hardcoding `true`
							// here (the previous behaviour) made every VSIX
							// Install appear under **Built-in** in the
							// Extensions sidebar and left `@installed` empty
							// because the default query filters for User-scope
							// extensions only.
							Description.IsBuiltin = !IsUserPath;

							// Boot-time exec-bit heal for user-scope extensions.
							// Runs only against `~/.fiddee/extensions/<id>/` (built-in
							// trees ship with correct modes from the bundle). Walks
							// `bin/`, `server/`, `tools/`, etc., promotes 0o644 →
							// 0o755 on files matching ELF / Mach-O / shebang magic.
							// One-shot per boot - cheap (a couple stat + read(4) calls
							// per file in those directories), and recovers extensions
							// installed before the in-extractor exec-bit fix landed
							// without forcing the user to reinstall.
							#[cfg(unix)]
							if IsUserPath {
								crate::ExtensionManagement::VsixInstaller::HealExecutableBits(&PotentialExtensionPath);
							}

							dev_log!(
								"ext-scan",
								"[ExtScan] accept path={} is_user={} is_builtin={} id={}",
								PotentialExtensionPath.display(),
								IsUserPath,
								Description.IsBuiltin,
								Description
									.Identifier
									.Get("value")
									.and_then(|V| V.as_str())
									.unwrap_or("<unknown>")
							);

							FoundExtensions.push(Description);
						},

						Err(error) => {
							parse_failures += 1;

							dev_log!(
								"extensions",
								"warn: [ExtensionScanner] Failed to parse package.json for extension at '{}': {}",
								PotentialExtensionPath.display(),
								error
							);

							dev_log!(
								"ext-scan",
								"[ExtScan] skip path={} reason=parse-failure err={}",
								PotentialExtensionPath.display(),
								error
							);
						},
					}
				},

				Err(error) => {
					missing_package_json += 1;

					dev_log!(
						"extensions",
						"warn: [ExtensionScanner] Could not read package.json at '{}': {}",
						PackageJsonPath.display(),
						error
					);

					dev_log!(
						"ext-scan",
						"[ExtScan] skip path={} reason=no-package-json err={}",
						PotentialExtensionPath.display(),
						error
					);
				},
			}
		}
	}

	dev_log!(
		"extensions",
		"[ExtensionScanner] Directory '{}' scan done: {} parsed, {} parse-failures, {} missing package.json, {} \
		 denied-dirs, {} test-extensions-skipped (Test={})",
		DirectoryPath.display(),
		FoundExtensions.len(),
		parse_failures,
		missing_package_json,
		denied_directory_count,
		test_extension_skips,
		AllowTestExtensions,
	);

	Ok(FoundExtensions)
}
