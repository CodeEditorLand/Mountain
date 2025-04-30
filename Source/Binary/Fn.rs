#![allow(non_snake_case)]

use std::{
	env,
	fs,
	path::{Path, PathBuf},
	sync::Arc,
};

pub fn Fn() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot build.")
		.block_on(async {
			let mut Build = tauri::Builder::default();

			#[cfg(any(windows, target_os = "linux"))]
			{
				Build = Build.any_thread();
			}

			Build
				.setup(|Tauri| {
					let mut Application = tauri::WebviewWindowBuilder::new(
						Tauri,
						"Application",
						tauri::WebviewUrl::App(PathBuf::from("Application/index.html")),
					)
					.use_https_scheme(true);

					#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
					{
						Application = Application.title("FIDDEE").maximized(true);
					}

					let Window = Application.build().expect("Cannot build Application window.");

					#[cfg(all(debug_assertions, not(any(target_os = "android", target_os = "ios"))))]
					{
						println!("Opening DevTools");

						Window.open_devtools();
					}

					Ok(())
				})
				.run(tauri::generate_context!())
				.expect("Error while running Tauri application");
		});
}
