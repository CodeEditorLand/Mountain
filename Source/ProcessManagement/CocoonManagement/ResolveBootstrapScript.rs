//! Resolve the Cocoon bootstrap script path (bundled resources first, then
//! dev layout relative to the executable) and pre-flight the Cocoon bundle
//! so a missing build fails fast with an actionable error.

use std::path::PathBuf;

use CommonLibrary::Error::CommonError::CommonError;
use tauri::{
	AppHandle,
	Manager,
	Wry,
	path::{BaseDirectory, PathResolver},
};

use crate::dev_log;

pub(crate) fn Fn(ApplicationHandle:&AppHandle) -> Result<PathBuf, CommonError> {
	let path_resolver:PathResolver<Wry> = ApplicationHandle.path().clone();

	// Resolve bootstrap script path.
	// 1) Try Tauri bundled resources (production builds).
	// 2) Fallback: resolve relative to the executable (dev builds). Dev layout:
	//    Target/debug/binary → ../../scripts/cocoon/bootstrap-fork.js
	let ScriptPath = path_resolver
		.resolve(super::BOOTSTRAP_SCRIPT_PATH, BaseDirectory::Resource)
		.ok()
		.filter(|P| P.exists())
		.or_else(|| {
			std::env::current_exe().ok().and_then(|Exe| {
				let MountainRoot = Exe.parent()?.parent()?.parent()?;

				let Candidate = MountainRoot.join(super::BOOTSTRAP_SCRIPT_PATH);

				if Candidate.exists() { Some(Candidate) } else { None }
			})
		})
		.ok_or_else(|| {
			CommonError::FileSystemNotFound(
				format!(
					"Cocoon bootstrap script '{}' not found in resources or relative to executable",
					super::BOOTSTRAP_SCRIPT_PATH
				)
				.into(),
			)
		})?;

	dev_log!(
		"cocoon",
		"[CocoonManagement] Found bootstrap script at: {}",
		ScriptPath.display()
	);

	crate::dev_log!("cocoon", "bootstrap script: {}", ScriptPath.display());

	// Pre-flight: Cocoon's bundle must exist or the spawned Node will
	// die silently on the first `import()` and we'll sit through 20+
	// seconds of `attempt N/M` retries with no diagnostic.
	//
	// Two layouts:
	//
	// 1. Bundle (.app): tauri.conf.json maps
	//    `Element/Cocoon/Target/Bootstrap/Implementation/Cocoon` →
	//    `Contents/Resources/Cocoon/Target/Bootstrap/Implementation/Cocoon`. The
	//    Tauri resource resolver finds it directly.
	//
	// 2. Repo (dev binary): bootstrap is at
	//    `Element/Mountain/scripts/cocoon/bootstrap-fork.js`, so walking `../../..`
	//    from the bootstrap dir reaches `Element/` and `COCOON_BUNDLE_PROBE`
	//    (`../Cocoon/Target/...`) descends into `Element/Cocoon/Target/...`.
	let BundleProbe = path_resolver
		.resolve("Cocoon/Target/Bootstrap/Implementation/Cocoon/Main.js", BaseDirectory::Resource)
		.ok()
		.filter(|P| P.exists());

	drop(path_resolver);

	if BundleProbe.is_none() {
		if let Some(BootstrapDirectory) = ScriptPath.parent() {
			let RepoProbePath = BootstrapDirectory.join("../..").join(super::COCOON_BUNDLE_PROBE);

			if !RepoProbePath.exists() {
				return Err(CommonError::IPCError {
					Description:format!(
						"Cocoon bundle is missing at {}. Run `pnpm run prepublishOnly \
						 --filter=@codeeditorland/cocoon` (or the full `./Maintain/Debug/Build.sh --profile \
						 debug-electron`) before launching - node will fail to import without it and Mountain will \
						 fall into degraded mode with zero extensions available. Root cause is typically an esbuild \
						 failure in an upstream Cocoon source file or a stale `rm -rf Element/Cocoon/Target` without \
						 a rebuild.",
						RepoProbePath.display()
					),
				});
			}

			dev_log!(
				"cocoon",
				"[CocoonManagement] pre-flight OK: bundle at {} (repo)",
				RepoProbePath.display()
			);
		}
	} else {
		dev_log!("cocoon", "[CocoonManagement] pre-flight OK: bundle in bundle resources");
	}

	Ok(ScriptPath)
}
